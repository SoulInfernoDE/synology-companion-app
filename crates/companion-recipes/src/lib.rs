//! Recipes wrap a community script or a set of commands so it can be run on a
//! NAS safely: the script comes from a pinned release and must match its
//! SHA-256; `check` changes nothing; a backup is taken before `apply`;
//! `restore` undoes it; every run is written to a journal on the NAS.
//!
//! Files on the NAS live in `~/.syno-companion/` of the signed-in account:
//! `recipes/<id>/<version>/` holds the scripts, `backups/` the backups and
//! `journal.jsonl` one line per run.

use std::time::{SystemTime, UNIX_EPOCH};

use companion_ssh::{Connection, Output, quote};
use serde::{Deserialize, Serialize};

/// The recipes that come with the app.
const BUILTIN: &[&str] = &[include_str!("../../../recipes/third-party-drives.toml")];

/// Exit status when the downloaded script does not match its checksum.
const CHECKSUM_MISMATCH: u32 = 44;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Ssh(#[from] companion_ssh::Error),
    #[error("the download of {0} does not match its checksum")]
    Checksum(String),
    #[error("sudo refused the password")]
    Sudo,
    #[error("{0} needs DSM {1} or newer")]
    Dsm(String, String),
    #[error("unknown option {0}")]
    Option(String),
    #[error("{0}")]
    Failed(String),
}

impl Error {
    pub fn key(&self) -> &'static str {
        match self {
            Self::Ssh(error) => error.key(),
            Self::Checksum(_) => "recipe-checksum",
            Self::Sudo => "sudo",
            Self::Dsm(..) => "recipe-dsm",
            Self::Option(_) | Self::Failed(_) => "recipe-failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Recipe {
    pub id: String,
    /// Repository of the script.
    pub source: String,
    pub author: String,
    pub license: String,
    /// The release the script is pinned to.
    pub version: String,
    pub download: String,
    pub file: String,
    pub sha256: String,
    /// Oldest DSM version, "7.0".
    pub dsm: String,
    /// Whether the change stays after a DSM update.
    pub survives_update: bool,
    /// Files and folders saved before `apply`.
    #[serde(default)]
    pub backup: Vec<String>,
    pub steps: Steps,
    #[serde(default)]
    pub options: Vec<RecipeOption>,
}

/// Shell commands; `{script}` is the script's path, `{options}` the chosen flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Steps {
    /// Exits with 0 when the recipe is applied, 1 when not. Changes nothing.
    pub check: String,
    pub apply: String,
    pub restore: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeOption {
    pub id: String,
    pub flag: String,
    #[serde(default)]
    pub default: bool,
}

/// The recipes that come with the app.
pub fn builtin() -> Vec<Recipe> {
    BUILTIN
        .iter()
        .map(|text| toml::from_str(text).expect("built-in recipe"))
        .collect()
}

/// What `check` says on a NAS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub applied: bool,
    /// The DSM version is new enough.
    pub supported: bool,
    /// "7.2.2-72806"
    pub dsm: String,
}

/// The end of an `apply` or `restore`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub ok: bool,
    /// What the script printed, without colour codes.
    pub output: String,
    /// Path of the backup on the NAS.
    pub backup: Option<String>,
    /// `check` after the run.
    pub applied: bool,
}

/// One line of the journal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// Seconds since 1970.
    pub time: u64,
    pub recipe: String,
    pub version: String,
    /// "apply" or "restore".
    pub action: String,
    #[serde(default)]
    pub options: Vec<String>,
    pub ok: bool,
    /// DSM version and build at the time.
    pub dsm: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backup: Option<String>,
    /// Which app and computer ran it.
    pub by: String,
}

/// Paths of the signed-in account's folder for recipes on the NAS.
struct Place {
    home: String,
}

impl Place {
    fn root(&self) -> String {
        format!("{}/.syno-companion", self.home)
    }

    fn scripts(&self, recipe: &Recipe) -> String {
        format!("{}/recipes/{}/{}", self.root(), recipe.id, recipe.version)
    }
}

async fn place(connection: &Connection) -> Result<Place, Error> {
    let output = connection.run(r#"printf '%s' "$HOME""#).await?;
    let home = output.stdout.trim().to_owned();
    if home.is_empty() || !home.starts_with('/') {
        return Err(companion_ssh::Error::NoHome.into());
    }
    Ok(Place { home })
}

/// "7.2.2-72806" from `/etc.defaults/VERSION`.
async fn dsm_version(connection: &Connection) -> Result<String, Error> {
    let output = connection
        .run(". /etc.defaults/VERSION 2>/dev/null; printf '%s-%s' \"$productversion\" \"$buildnumber\"")
        .await?;
    Ok(output.stdout.trim().to_owned())
}

fn version_parts(version: &str) -> Vec<u64> {
    version
        .split(|c: char| !c.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse().ok())
        .collect()
}

/// Whether DSM `dsm` ("7.2.2-72806") is at least `minimum` ("7.0").
fn supports(dsm: &str, minimum: &str) -> bool {
    let version = dsm.split('-').next().unwrap_or_default();
    version_parts(version) >= version_parts(minimum)
}

/// Removes ANSI escape sequences (colours, cursor moves) from script output.
fn plain(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                // Parameters and intermediates, then one final byte.
                for next in chars.by_ref() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
            continue;
        }
        if c != '\r' {
            result.push(c);
        }
    }
    result
}

/// The last `limit` characters, so a long log stays readable.
fn tail(text: &str, limit: usize) -> String {
    let count = text.chars().count();
    text.chars().skip(count.saturating_sub(limit)).collect()
}

impl Recipe {
    fn flags(&self, chosen: &[String]) -> Result<String, Error> {
        let mut flags = Vec::new();
        for id in chosen {
            let option = self
                .options
                .iter()
                .find(|option| option.id == *id)
                .ok_or_else(|| Error::Option(id.clone()))?;
            flags.push(quote(&option.flag));
        }
        Ok(flags.join(" "))
    }

    fn command(&self, template: &str, script: &str, flags: &str) -> String {
        template
            .replace("{script}", &quote(script))
            .replace("{options}", flags)
    }

    /// Runs `check` and compares the DSM version. Needs no root.
    pub async fn status(&self, connection: &Connection) -> Result<Status, Error> {
        let dsm = dsm_version(connection).await?;
        let check = connection.run(&self.steps.check).await?;
        Ok(Status {
            applied: check.success(),
            supported: supports(&dsm, &self.dsm),
            dsm,
        })
    }

    /// Downloads the script into the account's folder unless it is already
    /// there, and checks it against `sha256`.
    async fn fetch(&self, connection: &Connection, place: &Place) -> Result<String, Error> {
        let dir = place.scripts(self);
        let script = format!("{dir}/{}", self.file);
        let command = format!(
            r#"umask 077
mkdir -p {dir} && cd {dir} || exit 1
echo "{sha}  {file}" | sha256sum -c --status 2>/dev/null && exit 0
curl -fsSL --max-time 300 -o {part} {url} || exit 1
echo "{sha}  {part}" | sha256sum -c --status || {{ rm -f {part}; exit {CHECKSUM_MISMATCH}; }}
mv {part} {file}"#,
            dir = quote(&dir),
            sha = self.sha256,
            file = quote(&self.file),
            part = quote(&format!("{}.part", self.file)),
            url = quote(&self.download),
        );
        let output = connection.run(&command).await?;
        match output.status {
            Some(0) => Ok(script),
            Some(CHECKSUM_MISMATCH) => Err(Error::Checksum(self.file.clone())),
            _ => Err(Error::Failed(output.stderr.trim().to_owned())),
        }
    }

    /// Saves the `backup` paths that exist into a tarball the account owns.
    async fn back_up(
        &self,
        connection: &Connection,
        place: &Place,
        password: &str,
    ) -> Result<Option<String>, Error> {
        if self.backup.is_empty() {
            return Ok(None);
        }
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|time| time.as_secs())
            .unwrap_or_default();
        let dir = format!("{}/backups", place.root());
        let file = format!("{dir}/{}-{stamp}.tar.gz", self.id);
        let paths: Vec<String> = self.backup.iter().map(|path| quote(path)).collect();
        let command = format!(
            r#"set --
for p in {paths}; do [ -e "$p" ] && set -- "$@" "$p"; done
mkdir -p {dir} && tar czf {file} "$@" && chown "$SUDO_UID:$SUDO_GID" {dir} {file}"#,
            paths = paths.join(" "),
            dir = quote(&dir),
            file = quote(&file),
        );
        let output = connection.run_sudo(&command, password).await?;
        if output.sudo_refused() {
            return Err(Error::Sudo);
        }
        if !output.success() {
            return Err(Error::Failed(plain(output.stderr.trim())));
        }
        Ok(Some(file))
    }

    /// Runs a step as root, after checking the script once more.
    async fn run_as_root(
        &self,
        connection: &Connection,
        script: &str,
        step: &str,
        password: &str,
    ) -> Result<Output, Error> {
        let dir = script.rsplit_once('/').map(|(dir, _)| dir).unwrap_or(".");
        let command = format!(
            r#"cd {dir} || exit 1
echo "{sha}  {file}" | sha256sum -c --status || exit {CHECKSUM_MISMATCH}
{step} 2>&1"#,
            dir = quote(dir),
            sha = self.sha256,
            file = quote(&self.file),
        );
        let output = connection.run_sudo(&command, password).await?;
        if output.sudo_refused() {
            return Err(Error::Sudo);
        }
        if output.status == Some(CHECKSUM_MISMATCH) {
            return Err(Error::Checksum(self.file.clone()));
        }
        Ok(output)
    }

    /// Backs up, applies with the chosen options and checks the result.
    pub async fn apply(
        &self,
        connection: &Connection,
        options: &[String],
        password: &str,
        by: &str,
    ) -> Result<Run, Error> {
        let dsm = dsm_version(connection).await?;
        if !supports(&dsm, &self.dsm) {
            return Err(Error::Dsm(self.id.clone(), self.dsm.clone()));
        }
        let flags = self.flags(options)?;
        let place = place(connection).await?;
        let script = self.fetch(connection, &place).await?;
        let backup = self.back_up(connection, &place, password).await?;
        let step = self.command(&self.steps.apply, &script, &flags);
        let output = self
            .run_as_root(connection, &script, &step, password)
            .await?;
        let applied = connection.run(&self.steps.check).await?.success();
        let run = Run {
            ok: output.success(),
            output: tail(&plain(&output.stdout), 20_000),
            backup,
            applied,
        };
        self.record(connection, &place, "apply", options, &run, &dsm, by)
            .await?;
        Ok(run)
    }

    /// Undoes `apply` with the recipe's own restore step.
    pub async fn restore(
        &self,
        connection: &Connection,
        password: &str,
        by: &str,
    ) -> Result<Run, Error> {
        let dsm = dsm_version(connection).await?;
        let place = place(connection).await?;
        let script = self.fetch(connection, &place).await?;
        let step = self.command(&self.steps.restore, &script, "");
        let output = self
            .run_as_root(connection, &script, &step, password)
            .await?;
        let applied = connection.run(&self.steps.check).await?.success();
        let run = Run {
            ok: output.success() && !applied,
            output: tail(&plain(&output.stdout), 20_000),
            backup: None,
            applied,
        };
        self.record(connection, &place, "restore", &[], &run, &dsm, by)
            .await?;
        Ok(run)
    }

    #[allow(clippy::too_many_arguments)]
    async fn record(
        &self,
        connection: &Connection,
        place: &Place,
        action: &str,
        options: &[String],
        run: &Run,
        dsm: &str,
        by: &str,
    ) -> Result<(), Error> {
        let entry = Entry {
            time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|time| time.as_secs())
                .unwrap_or_default(),
            recipe: self.id.clone(),
            version: self.version.clone(),
            action: action.to_owned(),
            options: options.to_vec(),
            ok: run.ok,
            dsm: dsm.to_owned(),
            backup: run.backup.clone(),
            by: by.to_owned(),
        };
        let mut line =
            serde_json::to_string(&entry).map_err(|error| Error::Failed(error.to_string()))?;
        line.push('\n');
        let command = format!(
            "umask 077; mkdir -p {root} && cat >> {root}/journal.jsonl",
            root = quote(&place.root())
        );
        let output = connection.run_with_input(&command, line.as_bytes()).await?;
        if output.success() {
            Ok(())
        } else {
            Err(Error::Failed(output.stderr.trim().to_owned()))
        }
    }
}

/// The journal of the signed-in account on this NAS, oldest first.
pub async fn journal(connection: &Connection) -> Result<Vec<Entry>, Error> {
    let output = connection
        .run(r#"cat "$HOME/.syno-companion/journal.jsonl" 2>/dev/null"#)
        .await?;
    Ok(parse_journal(&output.stdout))
}

/// Lines that cannot be read are skipped, so one broken line hides nothing else.
pub fn parse_journal(text: &str) -> Vec<Entry> {
    text.lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_recipes_are_complete() {
        let recipes = builtin();
        assert!(!recipes.is_empty());
        for recipe in &recipes {
            assert_eq!(recipe.sha256.len(), 64, "{}", recipe.id);
            assert!(recipe.download.starts_with("https://"), "{}", recipe.id);
            assert!(recipe.download.contains(&recipe.version), "{}", recipe.id);
            assert!(recipe.steps.apply.contains("{script}"), "{}", recipe.id);
            assert!(recipe.steps.restore.contains("{script}"), "{}", recipe.id);
        }
    }

    #[test]
    fn options_become_quoted_flags() {
        let recipe = &builtin()[0];
        let flags = recipe.flags(&["noupdate".into(), "wdda".into()]).unwrap();
        assert_eq!(flags, "'--noupdate' '--wdda'");
        assert!(recipe.flags(&["rm -rf /".into()]).is_err());
        assert_eq!(
            recipe.command(&recipe.steps.apply, "/home/a b/x.sh", &flags),
            "bash '/home/a b/x.sh' -e '--noupdate' '--wdda'"
        );
    }

    #[test]
    fn compares_dsm_versions() {
        assert!(supports("7.2.2-72806", "7.0"));
        assert!(supports("7.0-41890", "7.0"));
        assert!(!supports("6.2.4-25556", "7.0"));
        assert!(!supports("", "7.0"));
    }

    #[test]
    fn strips_colours() {
        assert_eq!(plain("\u{1b}[0;31mERROR\u{1b}[0m done\r\n"), "ERROR done\n");
    }

    #[test]
    fn reads_the_journal_and_skips_broken_lines() {
        let text = r#"{"time":1,"recipe":"r","version":"v1","action":"apply","options":["noupdate"],"ok":true,"dsm":"7.2.2-72806","by":"x"}
not json
{"time":2,"recipe":"r","version":"v1","action":"restore","ok":false,"dsm":"7.2.2-72806","by":"x"}
"#;
        let entries = parse_journal(text);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].options, ["noupdate"]);
        assert!(!entries[1].ok);
    }
}
