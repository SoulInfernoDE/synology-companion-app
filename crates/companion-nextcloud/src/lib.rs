//! Nextcloud installed in Web Station: find it, check whether the PHP and
//! database versions of the NAS fit the next Nextcloud release, and update it
//! step by step over SSH.
//!
//! Every command runs as the owner of `config/config.php` (usually `http`),
//! with the PHP version that can run `occ`. The update is split into steps the
//! interface runs one after the other, so it can stop at any step, show what
//! happened and continue later; the state of a half-done update can be read
//! from `occ status`.

use companion_ssh::{Connection, Output, quote};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Ssh(#[from] companion_ssh::Error),
    #[error("sudo refused the password")]
    Sudo,
    #[error("{0}")]
    Failed(String),
}

impl Error {
    pub fn key(&self) -> &'static str {
        match self {
            Self::Ssh(error) => error.key(),
            Self::Sudo => "sudo",
            Self::Failed(_) => "nextcloud-failed",
        }
    }
}

/// Where Synology's packages put their programs; sudo's path lacks it.
const PATH: &str = "PATH=/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin";

/// One Nextcloud installation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Install {
    /// "/volume1/web/nextcloud"
    pub path: String,
    /// Owner of `config.php`; `occ` runs as this user.
    pub owner: String,
    /// The PHP binary that runs `occ`, "/usr/local/bin/php82".
    pub php: String,
    /// "8.2.20"
    pub php_version: String,
    /// "30.0.4"
    pub version: String,
    pub maintenance: bool,
    pub needs_db_upgrade: bool,
    pub database: Option<String>,
    pub redis: bool,
}

impl Install {
    /// Whether the fields that go into commands look like what `environment`
    /// finds, so an installation sent back by the interface cannot inject anything.
    pub fn valid(&self) -> bool {
        let simple = |text: &str| {
            !text.is_empty()
                && text
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
        };
        self.path.starts_with("/volume")
            && !self.path.contains("..")
            && simple(&self.owner)
            && self.php.starts_with("/usr/local/bin/php")
            && simple(&self.php["/usr/local/bin/".len().min(self.php.len())..])
    }
}

/// A folder of the signed-in account for the backups and notes of one update.
pub async fn work_folder(connection: &Connection, stamp: u64) -> Result<String, Error> {
    let output = connection.run(r#"printf '%s' "$HOME""#).await?;
    let home = output.stdout.trim();
    if !home.starts_with('/') {
        return Err(companion_ssh::Error::NoHome.into());
    }
    Ok(format!("{home}/.syno-companion/nextcloud/{stamp}"))
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub installs: Vec<Install>,
    /// PHP versions installed on the NAS, "8.2.20".
    pub php: Vec<String>,
    /// "10.11.6"
    pub mariadb: Option<String>,
    /// "7.2.5"
    pub redis: Option<String>,
}

fn sudo_checked(output: Output) -> Result<Output, Error> {
    if output.sudo_refused() {
        Err(Error::Sudo)
    } else {
        Ok(output)
    }
}

/// Finds installations below the `web` share and the versions around them.
const FIND: &str = r#"for occ in /volume*/web/occ /volume*/web/*/occ /volume*/web/*/*/occ; do
  [ -f "$occ" ] || continue
  dir=${occ%/occ}
  owner=$(stat -c %U "$dir/config/config.php" 2>/dev/null) || continue
  for php in $(ls -1 /usr/local/bin/php[0-9]* 2>/dev/null | sort -rV); do
    status=$(cd "$dir" && sudo -u "$owner" "$php" occ status --output=json 2>/dev/null | grep '^{')
    [ -n "$status" ] || continue
    config=$(cd "$dir" && sudo -u "$owner" "$php" -r '$CONFIG = []; include "config/config.php"; echo json_encode(["dbtype" => $CONFIG["dbtype"] ?? null, "redis" => isset($CONFIG["redis"])]);' 2>/dev/null)
    printf '@@install\n%s\n%s\n%s\n%s\n%s\n%s\n' "$dir" "$owner" "$php" "$("$php" -r 'echo PHP_VERSION;')" "$status" "$config"
    break
  done
done
echo @@php
for php in /usr/local/bin/php[0-9]*; do [ -x "$php" ] && "$php" -r 'echo PHP_VERSION, "\n";' 2>/dev/null; done
echo @@mariadb
/usr/local/mariadb10/bin/mysqld --version 2>/dev/null
echo @@redis
for r in /var/packages/redis/target/bin/redis-server /usr/local/bin/redis-server; do [ -x "$r" ] && "$r" --version && break; done
echo @@end"#;

/// "mysqld  Ver 10.11.6-MariaDB for Linux" → "10.11.6"
fn version_after(text: &str, marker: &str) -> Option<String> {
    let rest = &text[text.find(marker)? + marker.len()..];
    let version: String = rest
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    (!version.is_empty()).then_some(version)
}

pub(crate) fn parse_environment(text: &str) -> Environment {
    let mut environment = Environment::default();
    for block in text.split("@@").skip(1) {
        let mut lines = block.lines();
        let name = lines.next().unwrap_or_default().trim();
        let lines: Vec<&str> = lines.map(str::trim).filter(|l| !l.is_empty()).collect();
        match name {
            "install" => {
                let [path, owner, php, php_version, status, config, ..] = lines[..] else {
                    continue;
                };
                let status: serde_json::Value = serde_json::from_str(status).unwrap_or_default();
                let config: serde_json::Value = serde_json::from_str(config).unwrap_or_default();
                environment.installs.push(Install {
                    path: path.to_owned(),
                    owner: owner.to_owned(),
                    php: php.to_owned(),
                    php_version: php_version.to_owned(),
                    version: status["versionstring"]
                        .as_str()
                        .unwrap_or_default()
                        .to_owned(),
                    maintenance: status["maintenance"].as_bool().unwrap_or(false),
                    needs_db_upgrade: status["needsDbUpgrade"].as_bool().unwrap_or(false),
                    database: config["dbtype"].as_str().map(str::to_owned),
                    redis: config["redis"].as_bool().unwrap_or(false),
                });
            }
            "php" => {
                environment.php = lines.iter().map(|line| (*line).to_owned()).collect();
                environment.php.sort_by_key(|version| parts(version));
                environment.php.dedup();
            }
            "mariadb" => {
                environment.mariadb = lines.first().and_then(|line| version_after(line, "Ver"));
            }
            "redis" => {
                environment.redis = lines.first().and_then(|line| version_after(line, "v="));
            }
            _ => {}
        }
    }
    environment
}

fn parts(version: &str) -> Vec<u32> {
    version
        .split(|c: char| !c.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse().ok())
        .collect()
}

/// The PHP versions a Nextcloud release accepts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Requirements {
    /// The major version these are for, 31.
    pub major: u32,
    /// "8.1"
    pub php_min: String,
    /// The first PHP version that no longer works, "8.5".
    pub php_below: Option<String>,
}

impl Requirements {
    pub fn accepts_php(&self, version: &str) -> bool {
        let version = parts(version);
        let short = &version[..version.len().min(2)];
        short >= parts(&self.php_min).as_slice()
            && self
                .php_below
                .as_deref()
                .is_none_or(|below| short < parts(below).as_slice())
    }
}

/// `80100` → "8.1"
fn php_id(id: &str) -> Option<String> {
    let id: u32 = id.trim().parse().ok()?;
    Some(format!("{}.{}", id / 10000, (id / 100) % 100))
}

/// The bounds from Nextcloud's `lib/versioncheck.php`.
pub(crate) fn parse_versioncheck(major: u32, text: &str) -> Option<Requirements> {
    let bound = |marker: &str| {
        text.find(marker)
            .and_then(|at| php_id(text[at + marker.len()..].split(')').next()?))
    };
    Some(Requirements {
        major,
        php_min: bound("PHP_VERSION_ID < ")?,
        php_below: bound("PHP_VERSION_ID >= "),
    })
}

/// What `occ update:check` and the release branches say about the next version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    /// "31.0.5", when Nextcloud offers an update.
    pub available: Option<String>,
    /// For the version the update leads to (or the installed one).
    pub requirements: Option<Requirements>,
}

/// One step of the guided update, in the order they run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Step {
    /// Database dump and a copy of `config.php` into the account's folder.
    Backup,
    MaintenanceOn,
    /// Gives the owner of `config.php` every file it does not own yet, and
    /// remembers the previous owners.
    OpenPermissions,
    /// Nextcloud's own updater: download, check, replace.
    Update,
    /// `occ upgrade`
    Upgrade,
    /// `occ db:add-missing-indices` and friends.
    Repair,
    /// Gives the files back to their previous owners.
    ClosePermissions,
    MaintenanceOff,
}

impl Step {
    pub const ALL: [Step; 8] = [
        Step::Backup,
        Step::MaintenanceOn,
        Step::OpenPermissions,
        Step::Update,
        Step::Upgrade,
        Step::Repair,
        Step::ClosePermissions,
        Step::MaintenanceOff,
    ];
}

/// The shell that runs `occ` as the owner, in the installation folder.
fn occ(install: &Install, arguments: &str) -> String {
    format!(
        "cd {dir} && sudo -u {owner} {php} occ --no-interaction {arguments}",
        dir = quote(&install.path),
        owner = quote(&install.owner),
        php = quote(&install.php),
    )
}

pub(crate) fn step_script(install: &Install, step: Step, work: &str) -> String {
    let dir = quote(&install.path);
    let owner = quote(&install.owner);
    let php = quote(&install.php);
    let work = quote(work);
    match step {
        Step::Backup => format!(
            r#"set -e
mkdir -p {work}
cd {dir}
cp -p config/config.php {work}/config.php
sudo -u {owner} {php} -r '$CONFIG = []; include "config/config.php"; foreach (["dbtype", "dbhost", "dbuser", "dbpassword", "dbname"] as $k) {{ echo str_replace("\n", "", $CONFIG[$k] ?? ""), "\n"; }}' > {work}/.db
{{ read -r type; read -r host; read -r user; read -r pass; read -r name; }} < {work}/.db
rm -f {work}/.db
case "$type" in mysql) ;; *) echo "database $type: dump it with its own tools" >&2; exit 3 ;; esac
case "$host" in
  *:/*) set -- --socket="${{host#*:}}" ;;
  *:*) set -- -h "${{host%:*}}" -P "${{host##*:}}" ;;
  *) set -- -h "$host" ;;
esac
MYSQL_PWD="$pass" /usr/local/mariadb10/bin/mysqldump --single-transaction --quick "$@" -u "$user" "$name" | gzip > {work}/database.sql.gz
chown -R "$SUDO_UID:$SUDO_GID" {work}
ls -l {work}"#
        ),
        Step::MaintenanceOn => occ(install, "maintenance:mode --on"),
        Step::OpenPermissions => format!(
            r#"set -e
mkdir -p {work}
cd {dir}
find . ! -user {owner} -printf '%u:%g\t%p\n' > {work}/owners.tsv
count=$(wc -l < {work}/owners.tsv)
[ "$count" -eq 0 ] || find . ! -user {owner} -exec chown {owner} {{}} +
echo "$count"
chown "$SUDO_UID:$SUDO_GID" {work}/owners.tsv"#
        ),
        Step::Update => {
            format!("cd {dir} && sudo -u {owner} {php} updater/updater.phar --no-interaction")
        }
        Step::Upgrade => occ(install, "upgrade"),
        Step::Repair => format!(
            "{} && {} && {}",
            occ(install, "db:add-missing-indices"),
            occ(install, "db:add-missing-columns"),
            occ(install, "db:add-missing-primary-keys"),
        ),
        Step::ClosePermissions => format!(
            r#"[ -f {work}/owners.tsv ] || {{ echo 0; exit 0; }}
cd {dir}
count=0
while IFS='	' read -r owner path; do
  [ -e "$path" ] && chown "$owner" "$path" && count=$((count + 1))
done < {work}/owners.tsv
echo "$count""#
        ),
        Step::MaintenanceOff => occ(install, "maintenance:mode --off"),
    }
}

/// The end of a step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepRun {
    pub ok: bool,
    pub output: String,
}

fn tail(text: &str, limit: usize) -> String {
    let count = text.chars().count();
    text.chars().skip(count.saturating_sub(limit)).collect()
}

/// Every installation and the PHP, MariaDB and Redis versions of the NAS.
pub async fn environment(connection: &Connection, password: &str) -> Result<Environment, Error> {
    let script = format!("export {PATH}\n{FIND}");
    let output = sudo_checked(connection.run_sudo(&script, password).await?)?;
    Ok(parse_environment(&output.stdout))
}

/// Asks Nextcloud for an update and reads the PHP bounds of the release it
/// leads to from Nextcloud's sources on GitHub (fetched by the NAS).
pub async fn check(
    connection: &Connection,
    install: &Install,
    password: &str,
) -> Result<Check, Error> {
    let script = format!("export {PATH}\n{} 2>&1", occ(install, "update:check"));
    let output = sudo_checked(connection.run_sudo(&script, password).await?)?;
    let available = output
        .stdout
        .lines()
        .find_map(|line| version_after(line, "Nextcloud "))
        .filter(|version| parts(version) > parts(&install.version));
    let major = parts(available.as_deref().unwrap_or(&install.version))
        .first()
        .copied()
        .unwrap_or_default();
    let fetch = format!(
        "curl -fsSL --max-time 20 https://raw.githubusercontent.com/nextcloud/server/stable{major}/lib/versioncheck.php"
    );
    let source = connection.run(&fetch).await?;
    Ok(Check {
        available,
        requirements: source
            .success()
            .then(|| parse_versioncheck(major, &source.stdout))
            .flatten(),
    })
}

/// Runs one step of the update. `work` is a folder of the account for backups
/// and the list of previous owners, the same for every step of one update.
pub async fn run_step(
    connection: &Connection,
    install: &Install,
    step: Step,
    work: &str,
    password: &str,
) -> Result<StepRun, Error> {
    let script = format!("export {PATH}\n{}", step_script(install, step, work));
    let output = sudo_checked(
        connection
            .run_sudo(&format!("{{ {script}\n}} 2>&1"), password)
            .await?,
    )?;
    Ok(StepRun {
        ok: output.success(),
        output: tail(output.stdout.trim(), 20_000),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_installations_and_versions() {
        let text = "@@install\n/volume1/web/nextcloud\nhttp\n/usr/local/bin/php82\n8.2.20\n\
            {\"installed\":true,\"version\":\"30.0.4.1\",\"versionstring\":\"30.0.4\",\"maintenance\":false,\"needsDbUpgrade\":false}\n\
            {\"dbtype\":\"mysql\",\"redis\":true}\n\
            @@php\n8.2.20\n7.4.33\n8.1.29\n\
            @@mariadb\n/usr/local/mariadb10/bin/mysqld  Ver 10.11.6-MariaDB for Linux on x86_64\n\
            @@redis\nRedis server v=7.2.5 sha=00000000:0 malloc=libc bits=64\n@@end\n";
        let environment = parse_environment(text);
        assert_eq!(environment.installs.len(), 1);
        let install = &environment.installs[0];
        assert_eq!(install.version, "30.0.4");
        assert_eq!(install.php, "/usr/local/bin/php82");
        assert_eq!(install.database.as_deref(), Some("mysql"));
        assert!(install.redis);
        assert_eq!(environment.php, ["7.4.33", "8.1.29", "8.2.20"]);
        assert_eq!(environment.mariadb.as_deref(), Some("10.11.6"));
        assert_eq!(environment.redis.as_deref(), Some("7.2.5"));
    }

    #[test]
    fn reads_php_bounds() {
        let text = "if (PHP_VERSION_ID < 80200) {\n\techo 'requires at least PHP 8.2';\n}\n\
                    if (PHP_VERSION_ID >= 80600) {\n";
        let requirements = parse_versioncheck(33, text).unwrap();
        assert_eq!(requirements.php_min, "8.2");
        assert_eq!(requirements.php_below.as_deref(), Some("8.6"));
        assert!(requirements.accepts_php("8.2.20"));
        assert!(requirements.accepts_php("8.5.0"));
        assert!(!requirements.accepts_php("8.1.29"));
        assert!(!requirements.accepts_php("8.6.1"));
    }

    #[test]
    fn steps_run_as_the_owner_in_the_folder() {
        let install = Install {
            path: "/volume1/web/nextcloud".into(),
            owner: "http".into(),
            php: "/usr/local/bin/php82".into(),
            ..Install::default()
        };
        assert_eq!(
            step_script(&install, Step::MaintenanceOn, "/w"),
            "cd '/volume1/web/nextcloud' && sudo -u 'http' '/usr/local/bin/php82' occ --no-interaction maintenance:mode --on"
        );
        let backup = step_script(
            &install,
            Step::Backup,
            "/home/a/.syno-companion/nextcloud/1",
        );
        assert!(backup.contains("MYSQL_PWD=\"$pass\""));
        assert!(!backup.contains("-p\"$pass\""));
        assert!(
            step_script(&install, Step::Update, "/w")
                .contains("updater/updater.phar --no-interaction")
        );
        assert!(step_script(&install, Step::OpenPermissions, "/w").contains("! -user 'http'"));
    }

    #[test]
    fn rejects_installations_that_could_inject() {
        let good = Install {
            path: "/volume1/web/nextcloud".into(),
            owner: "http".into(),
            php: "/usr/local/bin/php82".into(),
            ..Install::default()
        };
        assert!(good.valid());
        let bad_owner = Install {
            owner: "http; rm -rf /".into(),
            ..good.clone()
        };
        assert!(!bad_owner.valid());
        let bad_php = Install {
            php: "/usr/local/bin/php82 -r x".into(),
            ..good.clone()
        };
        assert!(!bad_php.valid());
        let bad_path = Install {
            path: "/etc".into(),
            ..good
        };
        assert!(!bad_path.valid());
    }

    #[test]
    fn finds_versions_in_text() {
        assert_eq!(
            version_after(
                "Nextcloud 31.0.5 is available. Get more information",
                "Nextcloud "
            )
            .as_deref(),
            Some("31.0.5")
        );
        assert_eq!(version_after("Everything up to date", "Nextcloud "), None);
    }
}

#[cfg(test)]
mod shell {
    use super::*;

    /// Every generated script must at least parse; `sh -n` reads without running.
    #[test]
    fn scripts_parse() {
        let install = Install {
            path: "/volume1/web/next cloud".into(),
            owner: "http".into(),
            php: "/usr/local/bin/php82".into(),
            ..Install::default()
        };
        let mut scripts: Vec<String> = Step::ALL
            .iter()
            .map(|step| format!("export {PATH}\n{}", step_script(&install, *step, "/tmp/w")))
            .collect();
        scripts.push(FIND.to_owned());
        for script in scripts {
            let status = std::process::Command::new("sh")
                .args(["-n", "-c", &script])
                .status();
            if let Ok(status) = status {
                assert!(status.success(), "{script}");
            }
        }
    }
}
