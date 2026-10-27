//! SSH to the active NAS: linking a key of this app, diagnostics and the terminal.
//!
//! The DSM password is used once, to install the key; afterwards every connection
//! signs in with the key from the keychain and checks the host key the user trusted.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use companion_ssh::diagnostics::{Diagnostics, MemoryModule};
use companion_ssh::{Auth, Connection, ShellWriter};
use serde::{Deserialize, Serialize};
use tauri::State;
use tauri::async_runtime::Mutex as AsyncMutex;
use tauri::ipc::Channel;

use crate::commands::{AppState, Failure, keychain};
use crate::devices::{KnownDevice, SshLink};
use crate::secrets;

impl From<companion_ssh::Error> for Failure {
    fn from(error: companion_ssh::Error) -> Self {
        use companion_ssh::Error;
        let detail = match &error {
            Error::Network(message) | Error::Protocol(message) | Error::Key(message) => {
                Some(message.clone())
            }
            _ => None,
        };
        Failure::new(error.key(), detail)
    }
}

/// Open SSH connections per device, and the terminals running on them.
#[derive(Default)]
pub struct SshState {
    connections: AsyncMutex<HashMap<String, Arc<Connection>>>,
    terminals: Mutex<HashMap<u32, ShellWriter>>,
    next_terminal: AtomicU32,
}

impl SshState {
    /// Closes the connection to a device, e.g. when the user disconnects it.
    pub async fn drop_device(&self, id: &str) {
        self.connections.lock().await.remove(id);
    }

    fn terminal(&self, id: u32) -> Result<ShellWriter, Failure> {
        self.terminals
            .lock()
            .ok()
            .and_then(|terminals| terminals.get(&id).cloned())
            .ok_or_else(|| Failure::new("ssh-failed", "the terminal has ended".to_owned()))
    }
}

/// A connection to `device` with its key, reusing an open one.
async fn connection(ssh: &SshState, device: &KnownDevice) -> Result<Arc<Connection>, Failure> {
    let link = device
        .ssh
        .clone()
        .ok_or_else(|| Failure::new("ssh-not-linked", None))?;
    let mut connections = ssh.connections.lock().await;
    if let Some(open) = connections.get(&device.id).filter(|open| !open.is_closed()) {
        return Ok(open.clone());
    }
    let key = {
        let (id, account) = (device.id.clone(), device.account.clone());
        keychain(move || secrets::load_ssh_key(&id, &account))
            .await
            .flatten()
            .ok_or_else(|| Failure::new("ssh-not-linked", None))?
    };
    let connection = Connection::open(
        &device.host,
        link.port,
        &device.account,
        Auth::Key(key),
        Some(&link.host_key),
    )
    .await
    .map_err(|error| match error {
        companion_ssh::Error::AuthFailed => Failure::new("ssh-key-rejected", None),
        other => other.into(),
    })?;
    let connection = Arc::new(connection);
    connections.insert(device.id.clone(), connection.clone());
    Ok(connection)
}

async fn stored_password(device: &KnownDevice) -> Option<String> {
    let (id, account) = (device.id.clone(), device.account.clone());
    keychain(move || secrets::load(&id, &account).password)
        .await
        .flatten()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkRequest {
    port: u16,
    /// `None` uses the password from the keychain.
    password: Option<String>,
    /// Fingerprint of the host key the user just agreed to trust.
    trust_fingerprint: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum LinkOutcome {
    Linked { device: KnownDevice },
    PasswordRequired,
    UntrustedHostKey { fingerprint: String },
    HostKeyChanged { fingerprint: String },
    Failed(Failure),
}

/// Creates a key for this computer, installs it on the NAS with the DSM password
/// and checks that it works. The key goes to the keychain, the host key to the list.
#[tauri::command]
pub async fn ssh_link(
    request: LinkRequest,
    state: State<'_, AppState>,
    ssh: State<'_, SshState>,
) -> Result<LinkOutcome, Failure> {
    use companion_ssh::Error;

    let device = state.current_device().await?;
    let typed = request.password.is_some();
    let password = match request.password {
        Some(password) => password,
        None => match stored_password(&device).await {
            Some(password) => password,
            None => return Ok(LinkOutcome::PasswordRequired),
        },
    };
    let pin = request.trust_fingerprint.clone().or_else(|| {
        device
            .ssh
            .as_ref()
            .filter(|link| link.port == request.port)
            .map(|link| link.host_key.clone())
    });

    let with_password = match Connection::open(
        &device.host,
        request.port,
        &device.account,
        Auth::Password(password),
        pin.as_deref(),
    )
    .await
    {
        Ok(connection) => connection,
        Err(Error::UnknownHostKey { fingerprint }) => {
            return Ok(LinkOutcome::UntrustedHostKey { fingerprint });
        }
        Err(Error::HostKeyChanged { fingerprint }) => {
            return Ok(LinkOutcome::HostKeyChanged { fingerprint });
        }
        Err(Error::AuthFailed) if !typed => return Ok(LinkOutcome::PasswordRequired),
        Err(error) => return Ok(LinkOutcome::Failed(error.into())),
    };
    let Some(host_key) = pin else {
        return Ok(LinkOutcome::Failed(Failure::new("ssh-failed", None)));
    };

    let (private, public) = companion_ssh::generate_key(&key_comment())?;
    let installed = with_password.install_key(&public).await;
    with_password.close().await;
    if let Err(error) = installed {
        return Ok(LinkOutcome::Failed(error.into()));
    }

    // Only a key that signs in is worth keeping.
    let with_key = match Connection::open(
        &device.host,
        request.port,
        &device.account,
        Auth::Key(private.clone()),
        Some(&host_key),
    )
    .await
    {
        Ok(connection) => connection,
        Err(Error::AuthFailed) => {
            return Ok(LinkOutcome::Failed(Failure::new("ssh-key-rejected", None)));
        }
        Err(error) => return Ok(LinkOutcome::Failed(error.into())),
    };
    {
        let (id, account) = (device.id.clone(), device.account.clone());
        keychain(move || secrets::store_ssh_key(&id, &account, Some(&private)))
            .await
            .transpose()
            .map_err(|error| Failure::new("keychain", error))?;
    }
    let link = SshLink {
        port: request.port,
        host_key,
    };
    state.set_ssh(&device.id, Some(link.clone())).await?;
    ssh.connections
        .lock()
        .await
        .insert(device.id.clone(), Arc::new(with_key));
    Ok(LinkOutcome::Linked {
        device: KnownDevice {
            ssh: Some(link),
            ..device
        },
    })
}

/// The comment of the key on the NAS says where it came from.
fn key_comment() -> String {
    let system = match std::env::consts::OS {
        "macos" => "macos",
        "windows" => "windows",
        _ => "linux",
    };
    format!("syno-companion-{system}")
}

/// Takes the key off the NAS (when it can still be reached) and out of the keychain.
#[tauri::command]
pub async fn ssh_unlink(
    state: State<'_, AppState>,
    ssh: State<'_, SshState>,
) -> Result<(), Failure> {
    let device = state.current_device().await?;
    let key = {
        let (id, account) = (device.id.clone(), device.account.clone());
        keychain(move || secrets::load_ssh_key(&id, &account))
            .await
            .flatten()
    };
    if let (Some(key), Ok(open)) = (key, connection(&ssh, &device).await)
        && let Ok(public) = companion_ssh::public_key(&key)
    {
        let _ = open.remove_key(&public).await;
    }
    ssh.drop_device(&device.id).await;
    {
        let (id, account) = (device.id.clone(), device.account.clone());
        keychain(move || secrets::store_ssh_key(&id, &account, None))
            .await
            .transpose()
            .map_err(|error| Failure::new("keychain", error))?;
    }
    state.set_ssh(&device.id, None).await
}

#[tauri::command]
pub async fn diagnostics(
    state: State<'_, AppState>,
    ssh: State<'_, SshState>,
) -> Result<Diagnostics, Failure> {
    let device = state.current_device().await?;
    Ok(connection(&ssh, &device).await?.diagnostics().await?)
}

/// Reads the memory modules as root. Without a password from the interface or the
/// keychain it fails with `password-required`.
#[tauri::command]
pub async fn memory_modules(
    password: Option<String>,
    state: State<'_, AppState>,
    ssh: State<'_, SshState>,
) -> Result<Vec<MemoryModule>, Failure> {
    let device = state.current_device().await?;
    let password = match password {
        Some(password) => password,
        None => stored_password(&device)
            .await
            .ok_or_else(|| Failure::new("password-required", None))?,
    };
    let open = connection(&ssh, &device).await?;
    open.memory_modules(&password)
        .await
        .map_err(|error| match error {
            companion_ssh::Error::AuthFailed => Failure::new("sudo", None),
            other => other.into(),
        })
}

/// What a terminal sends to the interface.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum TerminalEvent {
    Output { data: Vec<u8> },
    Closed,
}

/// Opens a shell on the active NAS; its output streams through `events`.
#[tauri::command]
pub async fn terminal_open(
    cols: u32,
    rows: u32,
    events: Channel<TerminalEvent>,
    state: State<'_, AppState>,
    ssh: State<'_, SshState>,
) -> Result<u32, Failure> {
    let device = state.current_device().await?;
    let shell = connection(&ssh, &device).await?.shell(cols, rows).await?;
    let (writer, mut output) = shell.split();
    let id = ssh.next_terminal.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut terminals) = ssh.terminals.lock() {
        terminals.insert(id, writer);
    }
    tauri::async_runtime::spawn(async move {
        while let Some(data) = output.recv().await {
            if events.send(TerminalEvent::Output { data }).is_err() {
                break;
            }
        }
        let _ = events.send(TerminalEvent::Closed);
    });
    Ok(id)
}

#[tauri::command]
pub fn terminal_write(id: u32, data: String, ssh: State<'_, SshState>) -> Result<(), Failure> {
    if ssh.terminal(id)?.write(data.as_bytes()) {
        Ok(())
    } else {
        Err(Failure::new(
            "ssh-failed",
            "the terminal has ended".to_owned(),
        ))
    }
}

#[tauri::command]
pub fn terminal_resize(
    id: u32,
    cols: u32,
    rows: u32,
    ssh: State<'_, SshState>,
) -> Result<(), Failure> {
    ssh.terminal(id)?.resize(cols, rows);
    Ok(())
}

#[tauri::command]
pub fn terminal_close(id: u32, ssh: State<'_, SshState>) {
    if let Some(writer) = ssh
        .terminals
        .lock()
        .ok()
        .and_then(|mut terminals| terminals.remove(&id))
    {
        writer.close();
    }
}
