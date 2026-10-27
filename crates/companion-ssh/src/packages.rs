//! Installing and updating packages with DSM's own `synopkg`, as root.
//!
//! Synology's packages are installed by name from Synology's server. Packages of
//! other sources are downloaded on the NAS, checked against the MD5 checksum the
//! source publishes, and installed from the file; `synopkg install` also updates
//! an installed package.

use crate::{Connection, Error, Output, quote};

/// Where a package comes from.
pub enum Source<'a> {
    Synology,
    Download { url: &'a str, md5: &'a str },
}

/// Exit status of the install script when the download does not match its checksum.
const CHECKSUM_MISMATCH: u32 = 43;

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '+' | '-'))
}

fn script(id: &str, source: &Source<'_>) -> Result<String, Error> {
    if !valid_id(id) {
        return Err(Error::Protocol(format!("invalid package name {id:?}")));
    }
    Ok(match source {
        Source::Synology => format!("synopkg install_from_server {id}"),
        Source::Download { url, md5 } => {
            if md5.len() != 32 || !md5.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(Error::Protocol(format!("invalid checksum {md5:?}")));
            }
            if !url.starts_with("https://") && !url.starts_with("http://") {
                return Err(Error::Protocol(format!("invalid download address {url:?}")));
            }
            format!(
                r#"dir=$(mktemp -d /tmp/syno-companion.XXXXXX) || exit 1
trap 'rm -rf "$dir"' EXIT
curl -fsSL --max-time 900 -o "$dir/package.spk" {url} || exit 1
echo "{md5}  $dir/package.spk" | md5sum -c --status || exit {CHECKSUM_MISMATCH}
synopkg install "$dir/package.spk""#,
                url = quote(url),
            )
        }
    })
}

/// How an install ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Installed {
    Done,
    /// `sudo` did not accept the password.
    SudoRefused,
    ChecksumMismatch,
    /// `synopkg` or the download failed; the text says why.
    Failed(String),
}

fn outcome(output: &Output) -> Installed {
    if output.sudo_refused() {
        return Installed::SudoRefused;
    }
    if output.status == Some(CHECKSUM_MISMATCH) {
        return Installed::ChecksumMismatch;
    }
    // `synopkg install` answers in JSON; `install_from_server` only by its status.
    let reported = output
        .stdout
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .and_then(|line| line.find("\"success\":").map(|at| &line[at + 10..]))
        .map(|rest| rest.trim_start().starts_with("true"));
    match (output.success(), reported) {
        (true, None | Some(true)) => Installed::Done,
        _ => {
            let text = if output.stderr.trim().is_empty() {
                output.stdout.trim()
            } else {
                output.stderr.trim()
            };
            Installed::Failed(text.chars().take(2000).collect())
        }
    }
}

impl Output {
    /// Whether `sudo` refused the password (its messages are English on DSM).
    pub fn sudo_refused(&self) -> bool {
        self.status == Some(1)
            && [
                "incorrect password",
                "Sorry, try again",
                "no password was provided",
            ]
            .iter()
            .any(|message| self.stderr.contains(message))
    }
}

impl Connection {
    /// Installs a package, or updates it to the version its source offers.
    pub async fn install_package(
        &self,
        id: &str,
        source: Source<'_>,
        password: &str,
    ) -> Result<Installed, Error> {
        let script = script(id, &source)?;
        let output = self.run_sudo(&script, password).await?;
        Ok(outcome(&output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripts_check_their_input() {
        assert_eq!(
            script("TextEditor", &Source::Synology).unwrap(),
            "synopkg install_from_server TextEditor"
        );
        assert!(script("x; rm -rf /", &Source::Synology).is_err());
        let download = Source::Download {
            url: "https://packages.synocommunity.com/a.spk?arch=x&build=1",
            md5: "708543379fa661d5bfabb5d179ea7d42",
        };
        let text = script("synocli-file", &download).unwrap();
        assert!(text.contains("'https://packages.synocommunity.com/a.spk?arch=x&build=1'"));
        assert!(text.contains("708543379fa661d5bfabb5d179ea7d42  $dir/package.spk"));
        let bad = Source::Download {
            url: "https://x",
            md5: "nope",
        };
        assert!(script("a", &bad).is_err());
    }

    #[test]
    fn reads_how_an_install_ended() {
        let output = |status, stdout: &str, stderr: &str| Output {
            status: Some(status),
            stdout: stdout.into(),
            stderr: stderr.into(),
        };
        assert_eq!(outcome(&output(0, "", "")), Installed::Done);
        assert_eq!(
            outcome(&output(0, r#"{"error":{"code":0},"success":true}"#, "")),
            Installed::Done
        );
        assert!(matches!(
            outcome(&output(0, r#"{"error":{"code":313},"success":false}"#, "")),
            Installed::Failed(_)
        ));
        assert_eq!(
            outcome(&output(1, "", "Failed to query package list from server\n")),
            Installed::Failed("Failed to query package list from server".into())
        );
        assert_eq!(
            outcome(&output(
                1,
                "",
                "Sorry, try again.\nsudo: 1 incorrect password attempt\n"
            )),
            Installed::SudoRefused
        );
        assert_eq!(outcome(&output(43, "", "")), Installed::ChecksumMismatch);
    }
}
