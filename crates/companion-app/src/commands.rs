//! Commands the user interface calls through Tauri.

use std::sync::Mutex;
use std::time::Duration;

use companion_discovery::FoundNas;
use companion_dsm::{AuthError, Client, Credentials, Error, Overview, Session};
use serde::{Deserialize, Serialize};
use tauri::State;
use tauri::async_runtime::{Mutex as AsyncMutex, spawn_blocking};

use crate::devices::{DeviceStore, KnownDevice, device_id};
use crate::secrets::{self, StoredLogin};

/// How long the network is searched for NAS.
const DISCOVERY_TIME: Duration = Duration::from_secs(3);

struct Active {
    client: Client,
    session: Session,
    device: KnownDevice,
}

pub struct AppState {
    pub(crate) store: Mutex<DeviceStore>,
    active: AsyncMutex<Option<Active>>,
    /// The client holding DSM's setup session while the setup form is open.
    pub(crate) pending_setup: AsyncMutex<Option<crate::setup::PendingSetup>>,
}

impl AppState {
    pub fn new(store: DeviceStore) -> Self {
        Self {
            store: Mutex::new(store),
            active: AsyncMutex::new(None),
            pending_setup: AsyncMutex::new(None),
        }
    }
}

/// An error the interface shows: `key` picks the message, `detail` adds technical text.
#[derive(Debug, Serialize)]
pub struct Failure {
    key: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

impl Failure {
    pub(crate) fn new(key: &'static str, detail: impl Into<Option<String>>) -> Self {
        Self {
            key,
            detail: detail.into(),
        }
    }
}

impl From<Error> for Failure {
    fn from(error: Error) -> Self {
        let detail = match &error {
            Error::Network(message) | Error::Unexpected(message) => Some(message.clone()),
            Error::Api { .. } | Error::PermissionDenied(_) => Some(error.to_string()),
            _ => None,
        };
        Self::new(error.key(), detail)
    }
}

pub(crate) fn device_name() -> String {
    let system = match std::env::consts::OS {
        "macos" => "macOS",
        "windows" => "Windows",
        _ => "Linux",
    };
    format!("Syno Companion ({system})")
}

#[tauri::command]
pub async fn discover() -> Result<Vec<FoundNas>, Failure> {
    spawn_blocking(|| companion_discovery::browse(DISCOVERY_TIME))
        .await
        .map_err(|error| Failure::new("unexpected", error.to_string()))?
        .map_err(|error| Failure::new("discovery", error))
}

#[tauri::command]
pub fn known_devices(state: State<'_, AppState>) -> Vec<KnownDevice> {
    state
        .store
        .lock()
        .map(|store| store.all().to_vec())
        .unwrap_or_default()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRequest {
    host: String,
    port: u16,
    /// Name and model from discovery, shown until DSM reports its own.
    name: Option<String>,
    model: Option<String>,
    account: String,
    /// `None` uses the password from the keychain.
    password: Option<String>,
    otp: Option<String>,
    #[serde(default)]
    remember_device: bool,
    #[serde(default)]
    remember_login: bool,
    /// Fingerprint of the certificate the user just agreed to trust.
    trust_fingerprint: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ConnectOutcome {
    Connected { device: KnownDevice },
    PasswordRequired,
    OtpRequired,
    UntrustedCertificate { fingerprint: String },
    CertificateChanged { fingerprint: String },
    Failed(Failure),
}

pub(crate) async fn keychain<T: Send + 'static>(
    job: impl FnOnce() -> T + Send + 'static,
) -> Option<T> {
    spawn_blocking(job).await.ok()
}

#[tauri::command]
pub async fn connect(
    request: ConnectRequest,
    state: State<'_, AppState>,
) -> Result<ConnectOutcome, Failure> {
    let host = request.host.trim().to_owned();
    let id = device_id(&host, request.port);
    let account = request.account.trim().to_owned();
    let known = state
        .store
        .lock()
        .ok()
        .and_then(|store| store.get(&id).cloned());

    let stored = {
        let (id, account) = (id.clone(), account.clone());
        keychain(move || secrets::load(&id, &account))
            .await
            .unwrap_or_default()
    };
    let Some(password) = request.password.clone().or_else(|| stored.password.clone()) else {
        return Ok(ConnectOutcome::PasswordRequired);
    };
    let pin = request
        .trust_fingerprint
        .clone()
        .or_else(|| known.as_ref().and_then(|device| device.fingerprint.clone()));

    let client = Client::new(&host, request.port, pin.as_deref())?;
    let credentials = Credentials {
        account: account.clone(),
        password: password.clone(),
        otp: request.otp.clone().filter(|code| !code.trim().is_empty()),
        device_token: stored.device_token.clone(),
        remember_device: request.remember_device,
        device_name: device_name(),
    };
    let session = match client.login(&credentials).await {
        Ok(session) => session,
        Err(Error::UntrustedCertificate { fingerprint }) => {
            return Ok(ConnectOutcome::UntrustedCertificate { fingerprint });
        }
        Err(Error::CertificateChanged { fingerprint }) => {
            return Ok(ConnectOutcome::CertificateChanged { fingerprint });
        }
        Err(Error::Auth(AuthError::OtpRequired)) => return Ok(ConnectOutcome::OtpRequired),
        Err(Error::Auth(AuthError::WrongCredentials)) if request.password.is_none() => {
            // The stored password no longer works; ask for the current one.
            return Ok(ConnectOutcome::PasswordRequired);
        }
        Err(error) => return Ok(ConnectOutcome::Failed(error.into())),
    };

    // DSM's own server name and model win over what discovery or the list knew.
    let (dsm_name, dsm_model) = identity(&client, &session).await;
    let device = KnownDevice {
        id: id.clone(),
        name: dsm_name
            .or(request.name)
            .or_else(|| known.as_ref().map(|device| device.name.clone()))
            .unwrap_or_else(|| host.clone()),
        model: dsm_model
            .or(request.model)
            .or_else(|| known.as_ref().map(|device| device.model.clone()))
            .unwrap_or_default(),
        host,
        port: request.port,
        account: account.clone(),
        fingerprint: pin,
        remember_login: request.remember_login,
    };
    let remember_login = request.remember_login;
    let device = activate(&state, client, session, device, password, remember_login).await?;
    Ok(ConnectOutcome::Connected { device })
}

/// Server name and model from DSM; what cannot be read stays `None`.
pub(crate) async fn identity(
    client: &Client,
    session: &Session,
) -> (Option<String>, Option<String>) {
    let (name, model) = client.identity(session).await;
    // Development builds say why a part is missing, in the terminal that runs them.
    #[cfg(debug_assertions)]
    for (part, result) in [("server name", &name), ("model", &model)] {
        if let Err(error) = result {
            eprintln!("[identity] {part}: {error}");
        }
    }
    (name.ok(), model.ok())
}

/// Saves the device and its keychain entry, and makes the session the active one.
pub(crate) async fn activate(
    state: &AppState,
    client: Client,
    session: Session,
    device: KnownDevice,
    password: String,
    remember_login: bool,
) -> Result<KnownDevice, Failure> {
    let login = StoredLogin {
        password: remember_login.then_some(password),
        device_token: session.device_token.clone(),
    };
    {
        let (id, account) = (device.id.clone(), device.account.clone());
        // A keychain that is locked or missing must not block the sign-in.
        let _ = keychain(move || secrets::store(&id, &account, &login)).await;
    }
    if let Ok(mut store) = state.store.lock() {
        store
            .upsert(device.clone())
            .map_err(|error| Failure::new("storage", error.to_string()))?;
    }

    let previous = state.active.lock().await.replace(Active {
        client,
        session,
        device: device.clone(),
    });
    if let Some(previous) = previous {
        previous.client.logout(&previous.session).await;
    }
    Ok(device)
}

#[tauri::command]
pub async fn overview(state: State<'_, AppState>) -> Result<Overview, Failure> {
    let mut active = state.active.lock().await;
    let current = active
        .as_mut()
        .ok_or_else(|| Failure::new("not-connected", None))?;
    match current.client.overview(&current.session).await {
        Err(Error::SessionExpired) => {
            // DSM ends idle sessions; sign in again quietly when the keychain allows it.
            let (id, account) = (current.device.id.clone(), current.device.account.clone());
            let stored = keychain(move || secrets::load(&id, &account))
                .await
                .unwrap_or_default();
            let Some(password) = stored.password else {
                *active = None;
                return Err(Failure::new("session-expired", None));
            };
            let credentials = Credentials {
                account: current.device.account.clone(),
                password,
                device_token: stored.device_token,
                device_name: device_name(),
                ..Credentials::default()
            };
            match current.client.login(&credentials).await {
                Ok(session) => {
                    current.session = session;
                    Ok(current.client.overview(&current.session).await?)
                }
                Err(_) => {
                    *active = None;
                    Err(Failure::new("session-expired", None))
                }
            }
        }
        result => Ok(result?),
    }
}

#[tauri::command]
pub async fn disconnect(state: State<'_, AppState>) -> Result<(), Failure> {
    if let Some(active) = state.active.lock().await.take() {
        active.client.logout(&active.session).await;
    }
    Ok(())
}

/// Removes a NAS from the list and its entry from the keychain.
#[tauri::command]
pub async fn forget_device(id: String, state: State<'_, AppState>) -> Result<(), Failure> {
    let removed = state
        .store
        .lock()
        .map_err(|_| Failure::new("storage", None))?
        .remove(&id)
        .map_err(|error| Failure::new("storage", error.to_string()))?;
    if let Some(device) = removed {
        keychain(move || secrets::forget(&device.id, &device.account))
            .await
            .transpose()
            .map_err(|error| Failure::new("keychain", error))?;
    }
    Ok(())
}
