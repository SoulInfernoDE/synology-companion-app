//! Recipes (expert mode): their state on the active NAS, applying and restoring
//! them, and the journal of every run.

use companion_recipes::{Entry, Recipe, Run, Status};
use serde::Serialize;
use tauri::State;

use crate::commands::{AppState, Failure, device_name};
use crate::ssh::{SshState, connection, stored_password};

impl From<companion_recipes::Error> for Failure {
    fn from(error: companion_recipes::Error) -> Self {
        use companion_recipes::Error;
        let key = error.key();
        match error {
            Error::Ssh(error) => error.into(),
            Error::Failed(detail) | Error::Checksum(detail) | Error::Option(detail) => {
                Failure::new(key, detail)
            }
            other => Failure::new(key, other.to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeState {
    recipe: Recipe,
    /// `None` without SSH, or when the check could not run.
    status: Option<Status>,
}

/// The built-in recipes and, when SSH is set up, what their checks say.
#[tauri::command]
pub async fn recipes(
    state: State<'_, AppState>,
    ssh: State<'_, SshState>,
) -> Result<Vec<RecipeState>, Failure> {
    let device = state.current_device().await?;
    let open = match device.ssh {
        Some(_) => Some(connection(&ssh, &device).await?),
        None => None,
    };
    let mut result = Vec::new();
    for recipe in companion_recipes::builtin() {
        let status = match &open {
            Some(open) => Some(recipe.status(open).await?),
            None => None,
        };
        result.push(RecipeState { recipe, status });
    }
    Ok(result)
}

fn find(id: &str) -> Result<Recipe, Failure> {
    companion_recipes::builtin()
        .into_iter()
        .find(|recipe| recipe.id == id)
        .ok_or_else(|| Failure::new("recipe-failed", format!("unknown recipe {id}")))
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

#[tauri::command]
pub async fn recipe_apply(
    id: String,
    options: Vec<String>,
    password: Option<String>,
    state: State<'_, AppState>,
    ssh: State<'_, SshState>,
) -> Result<Run, Failure> {
    let recipe = find(&id)?;
    let device = state.current_device().await?;
    let password = self::password(password, &device).await?;
    let open = connection(&ssh, &device).await?;
    Ok(recipe
        .apply(&open, &options, &password, &device_name())
        .await?)
}

#[tauri::command]
pub async fn recipe_restore(
    id: String,
    password: Option<String>,
    state: State<'_, AppState>,
    ssh: State<'_, SshState>,
) -> Result<Run, Failure> {
    let recipe = find(&id)?;
    let device = state.current_device().await?;
    let password = self::password(password, &device).await?;
    let open = connection(&ssh, &device).await?;
    Ok(recipe.restore(&open, &password, &device_name()).await?)
}

#[tauri::command]
pub async fn journal(
    state: State<'_, AppState>,
    ssh: State<'_, SshState>,
) -> Result<Vec<Entry>, Failure> {
    let device = state.current_device().await?;
    let open = connection(&ssh, &device).await?;
    Ok(companion_recipes::journal(&open).await?)
}

/// Saves the journal as JSON where the user picks. Returns whether it was saved.
#[tauri::command]
pub async fn export_journal(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    ssh: State<'_, SshState>,
) -> Result<bool, Failure> {
    use tauri_plugin_dialog::DialogExt;

    let device = state.current_device().await?;
    let open = connection(&ssh, &device).await?;
    let entries = companion_recipes::journal(&open).await?;
    let text = serde_json::to_string_pretty(&serde_json::json!({
        "nas": device.name,
        "model": device.model,
        "entries": entries,
    }))
    .map_err(|error| Failure::new("unexpected", error.to_string()))?;

    let (sender, receiver) = tokio::sync::oneshot::channel();
    let name = format!("syno-companion-journal-{}.json", device.name);
    app.dialog()
        .file()
        .set_file_name(name)
        .add_filter("JSON", &["json"])
        .save_file(move |path| {
            let _ = sender.send(path);
        });
    let Some(path) = receiver.await.ok().flatten() else {
        return Ok(false);
    };
    let path = path
        .into_path()
        .map_err(|error| Failure::new("storage", error.to_string()))?;
    std::fs::write(path, text).map_err(|error| Failure::new("storage", error.to_string()))?;
    Ok(true)
}
