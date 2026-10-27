//! The Nextcloud section: installations in Web Station, the compatibility of
//! the next release and the guided update.

use std::time::{SystemTime, UNIX_EPOCH};

use companion_nextcloud::{Check, Environment, Install, Step, StepRun};
use tauri::State;

use crate::commands::{AppState, Failure};
use crate::ssh::{SshState, connection, stored_password};

impl From<companion_nextcloud::Error> for Failure {
    fn from(error: companion_nextcloud::Error) -> Self {
        use companion_nextcloud::Error;
        let key = error.key();
        match error {
            Error::Ssh(error) => error.into(),
            Error::Failed(detail) => Failure::new(key, detail),
            Error::Sudo => Failure::new(key, None),
        }
    }
}

async fn password(
    password: Option<String>,
    device: &crate::devices::KnownDevice,
) -> Result<String, Failure> {
    match password {
        Some(password) => Ok(password),
        None => stored_password(device)
            .await
            .ok_or_else(|| Failure::new("password-required", None)),
    }
}

fn checked(install: &Install) -> Result<(), Failure> {
    if install.valid() {
        Ok(())
    } else {
        Err(Failure::new(
            "nextcloud-failed",
            "invalid installation".to_owned(),
        ))
    }
}

#[tauri::command]
pub async fn nextcloud(
    password: Option<String>,
    state: State<'_, AppState>,
    ssh: State<'_, SshState>,
) -> Result<Environment, Failure> {
    let device = state.current_device().await?;
    let password = self::password(password, &device).await?;
    let open = connection(&ssh, &device).await?;
    Ok(companion_nextcloud::environment(&open, &password).await?)
}

#[tauri::command]
pub async fn nextcloud_check(
    install: Install,
    password: Option<String>,
    state: State<'_, AppState>,
    ssh: State<'_, SshState>,
) -> Result<Check, Failure> {
    checked(&install)?;
    let device = state.current_device().await?;
    let password = self::password(password, &device).await?;
    let open = connection(&ssh, &device).await?;
    Ok(companion_nextcloud::check(&open, &install, &password).await?)
}

/// A new folder for the backups of one update; every step of it uses the same.
#[tauri::command]
pub async fn nextcloud_work_folder(
    state: State<'_, AppState>,
    ssh: State<'_, SshState>,
) -> Result<String, Failure> {
    let device = state.current_device().await?;
    let open = connection(&ssh, &device).await?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|time| time.as_secs())
        .unwrap_or_default();
    Ok(companion_nextcloud::work_folder(&open, stamp).await?)
}

#[tauri::command]
pub async fn nextcloud_step(
    install: Install,
    step: Step,
    work: String,
    password: Option<String>,
    state: State<'_, AppState>,
    ssh: State<'_, SshState>,
) -> Result<StepRun, Failure> {
    checked(&install)?;
    if !work.starts_with('/')
        || work.contains("..")
        || !work.contains("/.syno-companion/nextcloud/")
    {
        return Err(Failure::new(
            "nextcloud-failed",
            "invalid work folder".to_owned(),
        ));
    }
    let device = state.current_device().await?;
    let password = self::password(password, &device).await?;
    let open = connection(&ssh, &device).await?;
    Ok(companion_nextcloud::run_step(&open, &install, step, &work, &password).await?)
}
