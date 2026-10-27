//! Passwords and DSM device tokens in the system keychain, one entry per NAS and account.
//! The keychain calls block, so callers run them on a blocking thread.

use serde::{Deserialize, Serialize};

const SERVICE: &str = "Syno Companion";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredLogin {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Lets DSM skip the authenticator code on this computer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_token: Option<String>,
}

fn entry(device_id: &str, account: &str) -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new(SERVICE, &format!("{account}@{device_id}"))
}

/// What is stored for this NAS and account; empty when nothing is or the keychain is unavailable.
pub fn load(device_id: &str, account: &str) -> StoredLogin {
    entry(device_id, account)
        .and_then(|entry| entry.get_password())
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

/// Stores `login`, or removes the entry when it holds nothing.
pub fn store(device_id: &str, account: &str, login: &StoredLogin) -> Result<(), String> {
    let entry = entry(device_id, account).map_err(|error| error.to_string())?;
    if login.password.is_none() && login.device_token.is_none() {
        return match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(error.to_string()),
        };
    }
    let json = serde_json::to_string(login).map_err(|error| error.to_string())?;
    entry
        .set_password(&json)
        .map_err(|error| error.to_string())?;
    // Secret Service tools such as Seahorse show the label; the library's own one
    // reads like an internal key.
    #[cfg(target_os = "linux")]
    {
        let label = format!("{SERVICE} – {account}@{device_id}");
        let _ = entry.update_attributes(&std::collections::HashMap::from([(
            "label",
            label.as_str(),
        )]));
    }
    Ok(())
}

pub fn forget(device_id: &str, account: &str) -> Result<(), String> {
    store(device_id, account, &StoredLogin::default())?;
    store_ssh_key(device_id, account, None)
}

fn ssh_entry(device_id: &str, account: &str) -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new(SERVICE, &format!("ssh:{account}@{device_id}"))
}

/// The private SSH key (OpenSSH format) this app made for the NAS and account.
pub fn load_ssh_key(device_id: &str, account: &str) -> Option<String> {
    ssh_entry(device_id, account)
        .and_then(|entry| entry.get_password())
        .ok()
}

/// Stores the private SSH key, or removes it with `None`.
pub fn store_ssh_key(device_id: &str, account: &str, key: Option<&str>) -> Result<(), String> {
    let entry = ssh_entry(device_id, account).map_err(|error| error.to_string())?;
    let Some(key) = key else {
        return match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(error.to_string()),
        };
    };
    entry.set_password(key).map_err(|error| error.to_string())?;
    #[cfg(target_os = "linux")]
    {
        let label = format!("{SERVICE} – SSH {account}@{device_id}");
        let _ = entry.update_attributes(&std::collections::HashMap::from([(
            "label",
            label.as_str(),
        )]));
    }
    Ok(())
}
