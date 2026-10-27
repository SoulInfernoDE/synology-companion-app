//! Commands the user interface calls through Tauri.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard};
use std::time::Duration;

use companion_discovery::FoundNas;
use companion_dsm::{
    AuthError, Client, Credentials, Error, Overview, Session, Shares, SshService, Updates,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use tauri::async_runtime::{Mutex as AsyncMutex, spawn_blocking};

use crate::devices::{DeviceStore, KnownDevice, SshLink, device_id};
use crate::i18n::Texts;
use crate::preferences::{self, Preferences};
use crate::secrets::{self, StoredLogin};
use crate::tray::TrayItems;

/// How long the network is searched for NAS.
const DISCOVERY_TIME: Duration = Duration::from_secs(3);

struct Active {
    client: Client,
    session: Session,
    device: KnownDevice,
}

/// The NAS the app is signed in to, and the one the interface shows.
#[derive(Default)]
struct Sessions {
    all: Vec<Active>,
    current: Option<String>,
}

/// What the interface shows in its NAS switcher.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Connected {
    devices: Vec<KnownDevice>,
    current: Option<String>,
}

impl Sessions {
    fn current_mut(&mut self) -> Option<&mut Active> {
        let id = self.current.as_deref()?;
        self.all.iter_mut().find(|active| active.device.id == id)
    }

    fn current(&self) -> Option<&Active> {
        let id = self.current.as_deref()?;
        self.all.iter().find(|active| active.device.id == id)
    }

    /// Adds a session (replacing one to the same NAS) and makes it current.
    fn insert(&mut self, active: Active) -> Option<Active> {
        let id = active.device.id.clone();
        let replaced = self
            .all
            .iter()
            .position(|known| known.device.id == id)
            .map(|index| self.all.remove(index));
        self.all.push(active);
        self.current = Some(id);
        replaced
    }

    /// Removes the current session; the first remaining one becomes current.
    fn remove_current(&mut self) -> Option<Active> {
        let id = self.current.take()?;
        let index = self.all.iter().position(|active| active.device.id == id)?;
        let removed = self.all.remove(index);
        self.current = self.all.first().map(|active| active.device.id.clone());
        Some(removed)
    }

    fn connected(&self) -> Connected {
        Connected {
            devices: self
                .all
                .iter()
                .map(|active| active.device.clone())
                .collect(),
            current: self.current.clone(),
        }
    }
}

pub struct AppState {
    pub(crate) store: Mutex<DeviceStore>,
    active: AsyncMutex<Sessions>,
    /// The client holding DSM's setup session while the setup form is open.
    pub(crate) pending_setup: AsyncMutex<Option<crate::setup::PendingSetup>>,
    config_dir: PathBuf,
    preferences: Mutex<Preferences>,
    texts: RwLock<Texts>,
}

/// A signed-in NAS, for background work that must not hold the lock while it waits.
pub struct Target {
    pub id: String,
    pub name: String,
    pub client: Client,
    pub session: Session,
}

impl AppState {
    pub fn new(config_dir: PathBuf) -> Self {
        let preferences = preferences::load(&config_dir);
        Self {
            store: Mutex::new(DeviceStore::load(&config_dir)),
            active: AsyncMutex::new(Sessions::default()),
            pending_setup: AsyncMutex::new(None),
            texts: RwLock::new(Texts::new(preferences.language.as_deref())),
            preferences: Mutex::new(preferences),
            config_dir,
        }
    }

    pub fn preferences(&self) -> MutexGuard<'_, Preferences> {
        self.preferences
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn texts(&self) -> RwLockReadGuard<'_, Texts> {
        self.texts
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The NAS the interface shows.
    pub(crate) async fn current_device(&self) -> Result<KnownDevice, Failure> {
        self.active
            .lock()
            .await
            .current()
            .map(|current| current.device.clone())
            .ok_or_else(|| Failure::new("not-connected", None))
    }

    /// Saves the SSH link of a device, in the list and in its session.
    pub(crate) async fn set_ssh(&self, id: &str, link: Option<SshLink>) -> Result<(), Failure> {
        self.store
            .lock()
            .map_err(|_| Failure::new("storage", None))?
            .set_ssh(id, link.clone())
            .map_err(|error| Failure::new("storage", error.to_string()))?;
        let mut active = self.active.lock().await;
        if let Some(session) = active
            .all
            .iter_mut()
            .find(|session| session.device.id == id)
        {
            session.device.ssh = link;
        }
        Ok(())
    }

    pub async fn targets(&self) -> Vec<Target> {
        self.active
            .lock()
            .await
            .all
            .iter()
            .map(|active| Target {
                id: active.device.id.clone(),
                name: active.device.name.clone(),
                client: active.client.clone(),
                session: active.session.clone(),
            })
            .collect()
    }
}

#[tauri::command]
pub fn get_preferences(state: State<'_, AppState>) -> Preferences {
    state.preferences().clone()
}

/// Saves the settings and applies a change of language to tray and notifications.
#[tauri::command]
pub fn set_preferences(
    preferences: Preferences,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), Failure> {
    use tauri::Manager;

    preferences::save(&state.config_dir, &preferences)
        .map_err(|error| Failure::new("storage", error.to_string()))?;
    let language_changed = state.preferences().language != preferences.language;
    *state.preferences() = preferences.clone();
    if language_changed {
        let texts = Texts::new(preferences.language.as_deref());
        if let Some(tray) = app.try_state::<TrayItems>() {
            tray.relabel(&texts);
        }
        if let Ok(mut current) = state.texts.write() {
            *current = texts;
        }
    }
    Ok(())
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
pub async fn discover(state: State<'_, AppState>) -> Result<Vec<FoundNas>, Failure> {
    let found = spawn_blocking(|| companion_discovery::browse(DISCOVERY_TIME))
        .await
        .map_err(|error| Failure::new("unexpected", error.to_string()))?
        .map_err(|error| Failure::new("discovery", error))?;
    // Saved NAS learn their MAC address from discovery, for Wake-on-LAN later.
    if let Ok(mut store) = state.store.lock() {
        for nas in found.iter().filter(|nas| nas.mac.is_some()) {
            let _ = store.remember_mac(&nas.host, nas.mac.as_deref().unwrap_or_default());
        }
    }
    Ok(found)
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
    /// MAC address from discovery, kept for Wake-on-LAN.
    mac: Option<String>,
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
        mac: request
            .mac
            .or_else(|| known.as_ref().and_then(|device| device.mac.clone())),
        ssh: known.as_ref().and_then(|device| device.ssh.clone()),
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

    let replaced = state.active.lock().await.insert(Active {
        client,
        session,
        device: device.clone(),
    });
    if let Some(replaced) = replaced {
        replaced.client.logout(&replaced.session).await;
    }
    Ok(device)
}

/// Signs in again after DSM ended an idle session, when the keychain has the
/// password. Returns whether a new session is in place.
async fn relogin(current: &mut Active) -> bool {
    let (id, account) = (current.device.id.clone(), current.device.account.clone());
    let stored = keychain(move || secrets::load(&id, &account))
        .await
        .unwrap_or_default();
    let Some(password) = stored.password else {
        return false;
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
            true
        }
        Err(_) => false,
    }
}

/// Runs a call on the active NAS. When DSM has ended the session, it signs in
/// again quietly and repeats the call once.
macro_rules! on_nas {
    ($state:expr, |$client:ident, $session:ident| $call:expr) => {{
        let mut active = $state.active.lock().await;
        let current = active
            .current_mut()
            .ok_or_else(|| Failure::new("not-connected", None))?;
        let first = {
            let ($client, $session) = (&current.client, &current.session);
            $call.await
        };
        match first {
            Err(Error::SessionExpired) => {
                if relogin(current).await {
                    let ($client, $session) = (&current.client, &current.session);
                    $call.await.map_err(Failure::from)
                } else {
                    active.remove_current();
                    Err(Failure::new("session-expired", None))
                }
            }
            other => other.map_err(Failure::from),
        }
    }};
}

#[tauri::command]
pub async fn overview(state: State<'_, AppState>) -> Result<Overview, Failure> {
    on_nas!(state, |client, session| client.overview(session))
}

#[tauri::command]
pub async fn updates(state: State<'_, AppState>) -> Result<Updates, Failure> {
    on_nas!(state, |client, session| async {
        Ok::<_, Error>(client.updates(session).await)
    })
}

#[tauri::command]
pub async fn shares(state: State<'_, AppState>) -> Result<Shares, Failure> {
    on_nas!(state, |client, session| client.shares(session))
}

#[tauri::command]
pub async fn stop_beep(state: State<'_, AppState>) -> Result<(), Failure> {
    on_nas!(state, |client, session| client.stop_beep(session))
}

#[tauri::command]
pub async fn ssh_service(state: State<'_, AppState>) -> Result<SshService, Failure> {
    on_nas!(state, |client, session| client.ssh_service(session))
}

#[tauri::command]
pub async fn enable_ssh(port: u16, state: State<'_, AppState>) -> Result<(), Failure> {
    on_nas!(state, |client, session| client.enable_ssh(session, port))
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Power {
    Reboot,
    Shutdown,
}

/// Restarts or shuts down the active NAS. The session ends with it.
#[tauri::command]
pub async fn power(action: Power, state: State<'_, AppState>) -> Result<(), Failure> {
    on_nas!(state, |client, session| async {
        match action {
            Power::Reboot => client.reboot(session).await,
            Power::Shutdown => client.shutdown(session).await,
        }
    })?;
    state.active.lock().await.remove_current();
    Ok(())
}

/// Opens a shared folder of the active NAS in the system's file manager over SMB.
#[tauri::command]
pub async fn open_share(
    name: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), Failure> {
    use tauri_plugin_opener::OpenerExt;

    let host = state
        .active
        .lock()
        .await
        .current()
        .map(|current| current.device.host.clone())
        .ok_or_else(|| Failure::new("not-connected", None))?;
    if name.is_empty() || name.contains(['/', '\\']) {
        return Err(Failure::new(
            "unexpected",
            format!("invalid share name {name:?}"),
        ));
    }
    // Windows opens UNC paths in Explorer; the other systems hand smb:// URLs to
    // their file manager (Finder, Nautilus, Nemo, Dolphin), which mounts the share.
    let result = if cfg!(windows) {
        app.opener()
            .open_path(format!(r"\\{host}\{name}"), None::<&str>)
    } else {
        app.opener()
            .open_url(format!("smb://{host}/{name}"), None::<&str>)
    };
    result.map_err(|error| Failure::new("open-share", error.to_string()))
}

/// Sends Wake-on-LAN to a saved NAS whose MAC address discovery has seen.
#[tauri::command]
pub fn wake(id: String, state: State<'_, AppState>) -> Result<(), Failure> {
    let device = state
        .store
        .lock()
        .ok()
        .and_then(|store| store.get(&id).cloned())
        .ok_or_else(|| Failure::new("unexpected", "unknown device".to_owned()))?;
    let mac = device
        .mac
        .ok_or_else(|| Failure::new("wake-no-mac", None))?;
    companion_discovery::wake(&mac, device.host.parse().ok())
        .map_err(|error| Failure::new("wake", error))
}

/// Signs out of the current NAS; another signed-in NAS becomes current.
#[tauri::command]
pub async fn disconnect(
    state: State<'_, AppState>,
    ssh: State<'_, crate::ssh::SshState>,
) -> Result<Connected, Failure> {
    let mut active = state.active.lock().await;
    if let Some(removed) = active.remove_current() {
        ssh.drop_device(&removed.device.id).await;
        removed.client.logout(&removed.session).await;
    }
    Ok(active.connected())
}

#[tauri::command]
pub async fn connected(state: State<'_, AppState>) -> Result<Connected, Failure> {
    Ok(state.active.lock().await.connected())
}

/// Makes another signed-in NAS the one the interface shows.
#[tauri::command]
pub async fn switch_to(id: String, state: State<'_, AppState>) -> Result<Connected, Failure> {
    let mut active = state.active.lock().await;
    if !active.all.iter().any(|session| session.device.id == id) {
        return Err(Failure::new("not-connected", None));
    }
    active.current = Some(id);
    Ok(active.connected())
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
