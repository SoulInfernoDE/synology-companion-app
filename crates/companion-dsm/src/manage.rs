//! Things the user does with a NAS beyond looking at it: updates, power, shared
//! folders. Every call here needs an account in the administrators group.

use serde::Serialize;
use serde_json::Value;

use crate::{Client, Error, Session};

/// A DSM version DSM offers to install.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DsmUpdate {
    /// "7.4.1-90080 Update 1".
    pub version: String,
    /// The update restarts the NAS.
    pub needs_reboot: bool,
}

/// A package with a newer version in Package Center.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageUpdate {
    pub id: String,
    pub name: String,
    pub installed: String,
    pub available: String,
}

/// What can be updated. `dsm_checked` false or `packages` `None` mean DSM could not
/// be asked (no internet on the NAS, missing permission).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Updates {
    pub dsm_checked: bool,
    /// `None` after a successful check: DSM is up to date.
    pub dsm: Option<DsmUpdate>,
    pub packages: Option<Vec<PackageUpdate>>,
}

/// A shared folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Share {
    pub name: String,
    pub description: String,
    /// The name to open over SMB: DSM serves the shared folder "homes" as "home",
    /// the user's own home folder.
    pub smb_name: String,
    pub encrypted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shares {
    pub shares: Vec<Share>,
    /// Whether the SMB service is on; `None` when DSM did not say.
    pub smb_enabled: Option<bool>,
}

/// Numeric parts of a package version: "4.15.13-0871" → [4, 15, 13, 871].
fn version_parts(version: &str) -> Vec<u64> {
    version
        .split(|c: char| !c.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse().ok())
        .collect()
}

/// Whether `available` is newer than `installed`.
pub(crate) fn is_newer(available: &str, installed: &str) -> bool {
    version_parts(available) > version_parts(installed)
}

fn packages(list: &Value) -> Vec<(String, String, String)> {
    list["packages"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(|package| {
            let id = package["id"]
                .as_str()
                .or_else(|| package["package"].as_str())?;
            let version = package["version"].as_str()?;
            let name = package["name"]
                .as_str()
                .or_else(|| package["dname"].as_str())
                .unwrap_or(id);
            Some((id.to_owned(), name.to_owned(), version.to_owned()))
        })
        .collect()
}

/// Installed packages that Package Center offers in a newer version, by name.
fn package_updates(installed: &Value, catalogue: &Value) -> Vec<PackageUpdate> {
    let offered = packages(catalogue);
    let mut updates: Vec<PackageUpdate> = packages(installed)
        .into_iter()
        .filter_map(|(id, name, version)| {
            let (_, _, available) = offered.iter().find(|(offer, _, _)| *offer == id)?;
            is_newer(available, &version).then(|| PackageUpdate {
                id,
                name,
                installed: version,
                available: available.clone(),
            })
        })
        .collect();
    updates.sort_by_key(|update| update.name.to_lowercase());
    updates
}

fn dsm_update(check: &Value) -> Option<DsmUpdate> {
    let update = &check["update"];
    if update["available"].as_bool() != Some(true) {
        return None;
    }
    let version = update["version"].as_str()?;
    Some(DsmUpdate {
        version: version.strip_prefix("DSM ").unwrap_or(version).to_owned(),
        needs_reboot: update["reboot"]
            .as_str()
            .is_some_and(|reboot| reboot != "none"),
    })
}

fn shares(list: &Value, smb: Option<&Value>) -> Shares {
    let shares = list["shares"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter(|share| share["hidden"].as_bool() != Some(true))
        .filter_map(|share| {
            let name = share["name"].as_str()?.to_owned();
            Some(Share {
                smb_name: if name == "homes" {
                    "home".into()
                } else {
                    name.clone()
                },
                description: share["desc"].as_str().unwrap_or_default().to_owned(),
                encrypted: share["encryption"].as_i64().is_some_and(|level| level > 0),
                name,
            })
        })
        .collect();
    Shares {
        shares,
        smb_enabled: smb.and_then(|smb| smb["enable_samba"].as_bool()),
    }
}

/// DSM's SSH service and what key sign-in needs from it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SshService {
    pub enabled: bool,
    pub port: u16,
    /// Key sign-in needs the user home service: `authorized_keys` lives in the home folder.
    pub homes: bool,
}

impl Client {
    /// Available DSM and package updates. Each part is asked on its own; one that
    /// fails is `None`.
    pub async fn updates(&self, session: &Session) -> Updates {
        let dsm = self
            .call(
                session,
                "SYNO.Core.Upgrade.Server",
                "check",
                3,
                &[("user_reading", "true"), ("need_auto_smallupdate", "true")],
            )
            .await
            .ok();
        let dsm_checked = dsm.is_some();
        let dsm = dsm.and_then(|check| dsm_update(&check));
        let installed = self
            .call(session, "SYNO.Core.Package", "list", 2, &[])
            .await;
        let catalogue = self
            .call(
                session,
                "SYNO.Core.Package.Server",
                "list",
                2,
                &[("blforcereload", "false"), ("blloadothers", "false")],
            )
            .await;
        let packages = match (installed, catalogue) {
            (Ok(installed), Ok(catalogue)) => Some(package_updates(&installed, &catalogue)),
            _ => None,
        };
        Updates {
            dsm_checked,
            dsm,
            packages,
        }
    }

    /// Restarts the NAS. DSM refuses while a task must not be interrupted.
    pub async fn reboot(&self, session: &Session) -> Result<(), Error> {
        self.call(
            session,
            "SYNO.Core.System",
            "reboot",
            1,
            &[("force", "false"), ("local", "false")],
        )
        .await
        .map(drop)
    }

    /// Shuts the NAS down. DSM refuses while a task must not be interrupted.
    pub async fn shutdown(&self, session: &Session) -> Result<(), Error> {
        self.call(
            session,
            "SYNO.Core.System",
            "shutdown",
            1,
            &[("force", "false"), ("local", "false")],
        )
        .await
        .map(drop)
    }

    /// Silences the beeper until the next event.
    pub async fn stop_beep(&self, session: &Session) -> Result<(), Error> {
        self.call(
            session,
            "SYNO.Core.Hardware.BeepControl",
            "stop_beep",
            1,
            &[],
        )
        .await
        .map(drop)
    }

    /// The shared folders this account sees, and whether SMB is on to open them.
    pub async fn ssh_service(&self, session: &Session) -> Result<SshService, Error> {
        let terminal = self
            .call(session, "SYNO.Core.Terminal", "get", 3, &[])
            .await?;
        // Accounts without the right to read it are no reason to fail.
        let homes = self
            .call(session, "SYNO.Core.User.Home", "get", 1, &[])
            .await
            .ok()
            .and_then(|home| home["enable"].as_bool())
            .unwrap_or(true);
        Ok(SshService {
            enabled: terminal["enable_ssh"].as_bool().unwrap_or(false),
            port: terminal["ssh_port"]
                .as_u64()
                .and_then(|port| u16::try_from(port).ok())
                .unwrap_or(22),
            homes,
        })
    }

    /// Turns on DSM's SSH service on `port`; Telnet stays as it is.
    pub async fn enable_ssh(&self, session: &Session, port: u16) -> Result<(), Error> {
        let port = port.to_string();
        self.call(
            session,
            "SYNO.Core.Terminal",
            "set",
            3,
            &[("enable_ssh", "true"), ("ssh_port", &port)],
        )
        .await
        .map(drop)
    }

    pub async fn shares(&self, session: &Session) -> Result<Shares, Error> {
        let list = self
            .call(
                session,
                "SYNO.Core.Share",
                "list",
                1,
                &[
                    ("shareType", "all"),
                    ("additional", r#"["hidden","encryption"]"#),
                ],
            )
            .await?;
        let smb = self
            .call(session, "SYNO.Core.FileServ.SMB", "get", 3, &[])
            .await
            .ok();
        Ok(shares(&list, smb.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn package_versions_compare_numerically() {
        assert!(is_newer("4.15.13-2503", "4.15.13-0871"));
        assert!(is_newer("1.0.10-0187", "1.0.6-0134"));
        assert!(!is_newer("1.4.2-1575", "1.4.2-1575"));
        assert!(!is_newer("1.0.9", "1.0.10"));
    }

    #[test]
    fn finds_package_updates() {
        let installed = json!({ "packages": [
            { "id": "SMBService", "name": "SMB Service", "version": "4.15.13-0871" },
            { "id": "FileStation", "name": "File Station", "version": "1.4.2-1575" },
            { "id": "Own", "name": "Own package", "version": "1.0" }
        ]});
        let catalogue = json!({ "packages": [
            { "id": "SMBService", "dname": "SMB Service", "version": "4.15.13-2503" },
            { "id": "FileStation", "dname": "File Station", "version": "1.4.2-1575" }
        ]});
        assert_eq!(
            package_updates(&installed, &catalogue),
            [PackageUpdate {
                id: "SMBService".into(),
                name: "SMB Service".into(),
                installed: "4.15.13-0871".into(),
                available: "4.15.13-2503".into(),
            }]
        );
    }

    #[test]
    fn reads_the_dsm_update() {
        let check = json!({ "update": { "available": true, "reboot": "now",
                                        "version": "DSM 7.4.1-90080 Update 1" } });
        assert_eq!(
            dsm_update(&check),
            Some(DsmUpdate {
                version: "7.4.1-90080 Update 1".into(),
                needs_reboot: true
            })
        );
        assert_eq!(
            dsm_update(&json!({ "update": { "available": false } })),
            None
        );
    }

    #[test]
    fn homes_opens_as_home_and_hidden_shares_stay_hidden() {
        let list = json!({ "shares": [
            { "name": "homes", "desc": "homes contains all users' home", "hidden": false, "encryption": 0 },
            { "name": "secret", "hidden": true },
            { "name": "vault", "desc": "", "hidden": false, "encryption": 1 }
        ]});
        let result = shares(&list, Some(&json!({ "enable_samba": true })));
        assert_eq!(result.smb_enabled, Some(true));
        assert_eq!(result.shares.len(), 2);
        assert_eq!(result.shares[0].smb_name, "home");
        assert!(result.shares[1].encrypted);
    }
}
