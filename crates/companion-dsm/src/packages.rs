//! Package Center: installed packages, the catalogue of Synology and of added
//! package sources (such as SynoCommunity), and the sources themselves.
//! Installing goes over SSH (`synopkg`), see `companion-ssh`.

use serde::Serialize;
use serde_json::Value;

use crate::manage::is_newer;
use crate::{Client, Error, Session};

/// One package, installed or offered or both.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Installed version.
    pub installed: Option<String>,
    /// Version Package Center offers.
    pub available: Option<String>,
    /// "SynoCommunity"; `None` for Synology's own packages.
    pub publisher: Option<String>,
    /// "running", "stop" for installed packages.
    pub status: Option<String>,
    pub beta: bool,
    /// Part of DSM; cannot be removed.
    pub system: bool,
    /// Packages this one needs, by id.
    pub depends: Vec<String>,
    /// Where a package from another source is downloaded, with its MD5 checksum.
    /// Synology's own packages are installed by name instead.
    pub download: Option<PackageDownload>,
}

impl Package {
    pub fn has_update(&self) -> bool {
        match (&self.installed, &self.available) {
            (Some(installed), Some(available)) => is_newer(available, installed),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageDownload {
    pub url: String,
    pub md5: String,
}

/// A package source added to Package Center.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageSources {
    pub feeds: Vec<Feed>,
    /// 0: Synology only, 1: Synology and trusted publishers, 2: any publisher.
    pub trust_level: u8,
}

fn text(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

/// Merges the installed list (`SYNO.Core.Package list`) with the catalogue
/// (`SYNO.Core.Package.Server list`), sorted by name.
pub(crate) fn merge(installed: &Value, catalogue: &Value) -> Vec<Package> {
    let mut packages: Vec<Package> = Vec::new();
    let offers = catalogue["packages"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    for offer in offers {
        let Some(id) = text(&offer["id"]).or_else(|| text(&offer["package"])) else {
            continue;
        };
        // The catalogue may list a package twice (stable and beta); the first wins.
        if packages.iter().any(|known| known.id == id) {
            continue;
        }
        let others = offer["source"].as_str() == Some("others");
        packages.push(Package {
            name: text(&offer["dname"]).unwrap_or_else(|| id.clone()),
            description: text(&offer["desc"]).unwrap_or_default(),
            installed: None,
            available: text(&offer["version"]),
            publisher: if others {
                text(&offer["distributor"]).or_else(|| text(&offer["maintainer"]))
            } else {
                None
            },
            status: None,
            beta: offer["beta"].as_bool().unwrap_or(false),
            system: false,
            depends: offer["deppkgs"]
                .as_object()
                .map(|deps| deps.keys().cloned().collect())
                .unwrap_or_default(),
            download: match (others, text(&offer["link"]), text(&offer["md5"])) {
                (true, Some(url), Some(md5)) => Some(PackageDownload { url, md5 }),
                _ => None,
            },
            id,
        });
    }
    let installed = installed["packages"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    for package in installed {
        let Some(id) = text(&package["id"]) else {
            continue;
        };
        let additional = &package["additional"];
        let index = match packages.iter().position(|known| known.id == id) {
            Some(index) => index,
            None => {
                packages.push(Package {
                    name: text(&package["name"]).unwrap_or_else(|| id.clone()),
                    description: String::new(),
                    installed: None,
                    available: None,
                    publisher: None,
                    status: None,
                    beta: false,
                    system: false,
                    depends: Vec::new(),
                    download: None,
                    id: id.clone(),
                });
                packages.len() - 1
            }
        };
        let entry = &mut packages[index];
        entry.installed = text(&package["version"]);
        if let Some(name) = text(&package["name"]) {
            entry.name = name;
        }
        if entry.description.is_empty() {
            entry.description = text(&additional["description"]).unwrap_or_default();
        }
        entry.status = text(&additional["status"]);
        entry.system = additional["install_type"].as_str() == Some("system");
        entry.beta |= additional["beta"].as_bool().unwrap_or(false);
    }
    packages.sort_by_key(|package| package.name.to_lowercase());
    packages
}

/// DSM wants `list` as JSON text inside a JSON string, the way its own pages send it.
fn double_encoded(value: &Value) -> String {
    Value::String(value.to_string()).to_string()
}

impl Client {
    /// Installed packages and everything Package Center offers, other sources included.
    pub async fn packages(&self, session: &Session) -> Result<Vec<Package>, Error> {
        let installed = self
            .call(
                session,
                "SYNO.Core.Package",
                "list",
                2,
                &[(
                    "additional",
                    r#"["description","status","beta","install_type"]"#,
                )],
            )
            .await?;
        // Without the catalogue (no internet on the NAS) the installed list still shows.
        let catalogue = self
            .call(
                session,
                "SYNO.Core.Package.Server",
                "list",
                2,
                &[("blforcereload", "false"), ("blloadothers", "true")],
            )
            .await
            .unwrap_or(Value::Null);
        Ok(merge(&installed, &catalogue))
    }

    pub async fn package_sources(&self, session: &Session) -> Result<PackageSources, Error> {
        let feeds = self
            .call(session, "SYNO.Core.Package.Feed", "list", 1, &[])
            .await?;
        let settings = self
            .call(session, "SYNO.Core.Package.Setting", "get", 1, &[])
            .await?;
        Ok(PackageSources {
            feeds: feeds["items"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or_default()
                .iter()
                .filter_map(|feed| {
                    Some(Feed {
                        name: text(&feed["name"]).unwrap_or_default(),
                        url: text(&feed["feed"])?,
                    })
                })
                .collect(),
            trust_level: settings["trust_level"]
                .as_u64()
                .and_then(|level| u8::try_from(level).ok())
                .unwrap_or(0),
        })
    }

    /// Adds a package source. DSM then also trusts the publisher's certificate.
    pub async fn add_feed(&self, session: &Session, name: &str, url: &str) -> Result<(), Error> {
        let list = double_encoded(&serde_json::json!({ "name": name, "feed": url }));
        self.call(
            session,
            "SYNO.Core.Package.Feed",
            "add",
            1,
            &[("list", &list)],
        )
        .await
        .map(drop)
    }

    pub async fn remove_feed(&self, session: &Session, url: &str) -> Result<(), Error> {
        let list = double_encoded(&serde_json::json!([url]));
        self.call(
            session,
            "SYNO.Core.Package.Feed",
            "delete",
            1,
            &[("list", &list)],
        )
        .await
        .map(drop)
    }

    /// Sets who may publish packages: 0 Synology only, 1 also trusted publishers.
    pub async fn set_trust_level(&self, session: &Session, level: u8) -> Result<(), Error> {
        let level = level.to_string();
        self.call(
            session,
            "SYNO.Core.Package.Setting",
            "set",
            1,
            &[("trust_level", &level)],
        )
        .await
        .map(drop)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn merges_installed_and_offered_packages() {
        let installed = json!({ "packages": [
            { "id": "SMBService", "name": "SMB Service", "version": "4.15.13-0871",
              "additional": { "status": "running", "install_type": "system" } },
            { "id": "synocli-file", "name": "SynoCli File Tools", "version": "4.0.3-31",
              "additional": { "status": "stop", "install_type": "" } },
            { "id": "Private", "name": "Private", "version": "1.0" }
        ]});
        let catalogue = json!({ "packages": [
            { "id": "synocli-file", "dname": "SynoCli File Tools", "version": "4.0.3-32",
              "source": "others", "distributor": "SynoCommunity",
              "link": "https://example.org/synocli-file.spk", "md5": "abc", "deppkgs": null },
            { "id": "TextEditor", "dname": "Text Editor", "version": "1.2.5-0254",
              "source": "syno", "deppkgs": { "Python3": "" } },
            { "id": "TextEditor", "dname": "Text Editor", "version": "1.3.0-0001", "beta": true }
        ]});
        let packages = merge(&installed, &catalogue);
        let names: Vec<_> = packages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "Private",
                "SMB Service",
                "SynoCli File Tools",
                "Text Editor"
            ]
        );

        let synocli = &packages[2];
        assert!(synocli.has_update());
        assert_eq!(synocli.publisher.as_deref(), Some("SynoCommunity"));
        assert_eq!(
            synocli.download.as_ref().map(|d| d.md5.as_str()),
            Some("abc")
        );
        assert_eq!(synocli.status.as_deref(), Some("stop"));

        let editor = &packages[3];
        assert_eq!(editor.installed, None);
        assert_eq!(editor.available.as_deref(), Some("1.2.5-0254"));
        assert_eq!(editor.depends, ["Python3"]);
        assert_eq!(editor.download, None);
        assert!(!editor.beta);

        assert!(packages[1].system);
        assert!(!packages[0].has_update());
    }

    #[test]
    fn lists_are_sent_as_json_text_in_a_string() {
        assert_eq!(
            double_encoded(&json!(["https://packages.synocommunity.com/"])),
            r#""[\"https://packages.synocommunity.com/\"]""#
        );
    }
}
