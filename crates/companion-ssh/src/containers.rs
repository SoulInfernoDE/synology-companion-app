//! Containers of Container Manager: which run an older image than the one their
//! tag points to, and recreating them. Docker needs root on DSM.
//!
//! Checking pulls each image tag again (`docker pull`), then compares the image
//! a container runs with the image the tag now names. Containers of a Container
//! Manager project (Docker Compose) are recreated with `docker compose up -d`;
//! a single container is recreated the same way only when it belongs to one.

use serde::Serialize;

use crate::{Connection, Error, Output, quote};

/// Container Manager installs the docker command outside sudo's default path.
const PATH: &str = "PATH=/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    pub name: String,
    /// "nginx:latest"
    pub image: String,
    /// "running", "exited"
    pub state: String,
    /// Compose project and its folder, for containers of a Container Manager project.
    pub project: Option<String>,
    pub project_dir: Option<String>,
    /// The tag now names a newer image than the one the container runs.
    pub update: bool,
}

/// How a run as root ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome<T> {
    Done(T),
    SudoRefused,
    /// Docker is not installed or not running.
    NoDocker,
    Failed(String),
}

fn failed(output: &Output) -> String {
    let text = if output.stderr.trim().is_empty() {
        &output.stdout
    } else {
        &output.stderr
    };
    text.trim().chars().take(2000).collect()
}

/// Lists containers and, with `pull`, fetches every tag first.
fn script(pull: bool) -> String {
    let pull = if pull {
        r#"docker ps -a --format '{{.Image}}' | sort -u | while read -r image; do
  case "$image" in *@sha256:*|sha256:*) continue ;; esac
  docker pull -q "$image" >/dev/null 2>&1
done
"#
    } else {
        ""
    };
    format!(
        r#"export {PATH}
command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1 || exit 42
{pull}docker ps -a -q | while read -r id; do
  docker inspect -f '{{{{.Name}}}}	{{{{.Config.Image}}}}	{{{{.State.Status}}}}	{{{{.Image}}}}	{{{{index .Config.Labels "com.docker.compose.project"}}}}	{{{{index .Config.Labels "com.docker.compose.project.working_dir"}}}}' "$id"
done | while IFS='	' read -r name image state running project dir; do
  current=$(docker image inspect -f '{{{{.Id}}}}' "$image" 2>/dev/null)
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "${{name#/}}" "$image" "$state" "$project" "$dir" "$([ -n "$current" ] && [ "$current" != "$running" ] && echo yes || echo no)"
done"#
    )
}

/// One tab-separated line per container, as `script` prints them.
pub(crate) fn parse(text: &str) -> Vec<Container> {
    let mut containers: Vec<Container> = text
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            let [name, image, state, project, dir, update] = fields[..] else {
                return None;
            };
            let some = |value: &str| {
                let value = value.trim();
                (!value.is_empty() && value != "<no value>").then(|| value.to_owned())
            };
            Some(Container {
                name: name.to_owned(),
                image: image.to_owned(),
                state: state.to_owned(),
                project: some(project),
                project_dir: some(dir),
                update: update.trim() == "yes",
            })
        })
        .collect();
    containers.sort_by(|a, b| (&a.project, &a.name).cmp(&(&b.project, &b.name)));
    containers
}

impl Connection {
    async fn docker(&self, script: &str, password: &str) -> Result<Outcome<Output>, Error> {
        let output = self.run_sudo(script, password).await?;
        Ok(if output.sudo_refused() {
            Outcome::SudoRefused
        } else if output.status == Some(42) {
            Outcome::NoDocker
        } else {
            Outcome::Done(output)
        })
    }

    /// The containers; with `pull`, after fetching every image tag again.
    pub async fn containers(
        &self,
        pull: bool,
        password: &str,
    ) -> Result<Outcome<Vec<Container>>, Error> {
        Ok(match self.docker(&script(pull), password).await? {
            Outcome::Done(output) if output.success() => Outcome::Done(parse(&output.stdout)),
            Outcome::Done(output) => Outcome::Failed(failed(&output)),
            Outcome::SudoRefused => Outcome::SudoRefused,
            Outcome::NoDocker => Outcome::NoDocker,
            Outcome::Failed(reason) => Outcome::Failed(reason),
        })
    }

    /// Recreates the containers of a Compose project with the images their tags
    /// name now. Volumes and settings stay, as they are part of the project.
    pub async fn update_project(
        &self,
        project: &str,
        dir: &str,
        password: &str,
    ) -> Result<Outcome<String>, Error> {
        if !dir.starts_with('/') {
            return Err(Error::Protocol(format!("invalid project folder {dir:?}")));
        }
        let script = format!(
            r#"export {PATH}
command -v docker >/dev/null 2>&1 || exit 42
cd {dir} || exit 1
docker compose -p {project} pull 2>&1 && docker compose -p {project} up -d --remove-orphans 2>&1"#,
            dir = quote(dir),
            project = quote(project),
        );
        Ok(match self.docker(&script, password).await? {
            Outcome::Done(output) if output.success() => {
                Outcome::Done(output.stdout.trim().to_owned())
            }
            Outcome::Done(output) => Outcome::Failed(failed(&output)),
            Outcome::SudoRefused => Outcome::SudoRefused,
            Outcome::NoDocker => Outcome::NoDocker,
            Outcome::Failed(reason) => Outcome::Failed(reason),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_containers() {
        let text = "web\tnginx:latest\trunning\tsite\t/volume1/docker/site\tyes\n\
                    db\tmariadb:11\trunning\tsite\t/volume1/docker/site\tno\n\
                    alone\tpihole/pihole\texited\t<no value>\t<no value>\tno\n\
                    broken line\n";
        let containers = parse(text);
        assert_eq!(containers.len(), 3);
        assert_eq!(containers[0].name, "alone");
        assert_eq!(containers[0].project, None);
        assert_eq!(containers[1].name, "db");
        assert!(!containers[1].update);
        assert!(containers[2].update);
        assert_eq!(
            containers[2].project_dir.as_deref(),
            Some("/volume1/docker/site")
        );
    }

    #[test]
    fn the_script_escapes_go_templates() {
        let text = script(true);
        assert!(text.contains("{{.Config.Image}}"));
        assert!(text.contains("docker pull -q"));
        assert!(text.contains("${name#/}"));
        assert!(!script(false).contains("docker pull"));
    }
}
