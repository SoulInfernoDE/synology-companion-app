//! First-time setup of a NAS whose DSM welcome wizard has not run yet. The user
//! fills in the form in the app; nothing changes on the NAS before they confirm it.

use companion_dsm::{Client, Error, SetupInfo, SetupRequest, SetupState, UpdatePolicy};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::{AppState, Failure, activate, device_name, identity};
use crate::devices::{KnownDevice, device_id};

/// DSM's setup session, kept from opening the form until it is sent.
pub struct PendingSetup {
    id: String,
    client: Client,
    fingerprint: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeRequest {
    host: String,
    port: u16,
    /// Fingerprint of the certificate the user just agreed to trust.
    trust_fingerprint: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ProbeOutcome {
    Configured,
    NotConfigured,
    UntrustedCertificate { fingerprint: String },
    CertificateChanged { fingerprint: String },
    Failed(Failure),
}

/// The certificate to accept: the one just trusted, or the one pinned earlier.
fn pin_for(state: &AppState, id: &str, trusted: Option<String>) -> Option<String> {
    trusted.or_else(|| {
        state
            .store
            .lock()
            .ok()
            .and_then(|store| store.get(id).and_then(|device| device.fingerprint.clone()))
    })
}

/// Finds out, without signing in, whether DSM's welcome wizard still has to run.
#[tauri::command]
pub async fn probe(
    request: ProbeRequest,
    state: State<'_, AppState>,
) -> Result<ProbeOutcome, Failure> {
    let host = request.host.trim();
    let id = device_id(host, request.port);
    let pin = pin_for(&state, &id, request.trust_fingerprint);
    let client = Client::new(host, request.port, pin.as_deref())?;
    Ok(match client.setup_state().await {
        Ok(SetupState::Configured) => ProbeOutcome::Configured,
        Ok(SetupState::NotConfigured) => ProbeOutcome::NotConfigured,
        Err(Error::UntrustedCertificate { fingerprint }) => {
            ProbeOutcome::UntrustedCertificate { fingerprint }
        }
        Err(Error::CertificateChanged { fingerprint }) => {
            ProbeOutcome::CertificateChanged { fingerprint }
        }
        Err(error) => ProbeOutcome::Failed(error.into()),
    })
}

/// Opens DSM's setup session and returns what the setup form needs.
#[tauri::command]
pub async fn begin_setup(
    request: ProbeRequest,
    state: State<'_, AppState>,
) -> Result<SetupInfo, Failure> {
    let host = request.host.trim();
    let id = device_id(host, request.port);
    let pin = pin_for(&state, &id, request.trust_fingerprint);
    let client = Client::new(host, request.port, pin.as_deref())?;
    let info = client.begin_setup().await?;
    *state.pending_setup.lock().await = Some(PendingSetup {
        id,
        client,
        fingerprint: pin,
    });
    Ok(info)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteSetupRequest {
    host: String,
    port: u16,
    model: Option<String>,
    device_name: String,
    account: String,
    password: String,
    time_zone: String,
    #[serde(default)]
    updates: UpdatePolicy,
    #[serde(default)]
    remember_login: bool,
    /// MAC address from discovery, kept for Wake-on-LAN.
    mac: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupFinished {
    device: KnownDevice,
    /// Optional settings DSM refused; the setup itself is done.
    skipped: Vec<String>,
}

/// Sets the NAS up with the user's account and signs in with it.
#[tauri::command]
pub async fn complete_setup(
    request: CompleteSetupRequest,
    state: State<'_, AppState>,
) -> Result<SetupFinished, Failure> {
    let host = request.host.trim().to_owned();
    let id = device_id(&host, request.port);
    let pending = state
        .pending_setup
        .lock()
        .await
        .take()
        .filter(|pending| pending.id == id)
        .ok_or_else(|| Failure::new("setup-expired", None))?;

    let account = request.account.trim().to_owned();
    let done = pending
        .client
        .complete_setup(&SetupRequest {
            device_name: request.device_name.trim().to_owned(),
            account: account.clone(),
            password: request.password.clone(),
            time_zone: request.time_zone,
            updates: request.updates,
            device_label: device_name(),
        })
        .await?;

    let (_, dsm_model) = identity(&pending.client, &done.session).await;
    let device = KnownDevice {
        id,
        name: request.device_name.trim().to_owned(),
        model: dsm_model.or(request.model).unwrap_or_default(),
        host,
        port: request.port,
        account,
        fingerprint: pending.fingerprint,
        remember_login: request.remember_login,
        mac: request.mac,
    };
    let device = activate(
        &state,
        pending.client,
        done.session,
        device,
        request.password,
        request.remember_login,
    )
    .await?;
    Ok(SetupFinished {
        device,
        skipped: done.skipped,
    })
}
