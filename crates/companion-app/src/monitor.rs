//! Watches every signed-in NAS in the background, also while the window is hidden:
//! the tray shows their state, and notifications tell about problems, about a NAS
//! not answering and about a new DSM version.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use companion_dsm::{Error, Health, Overview};
use fluent_bundle::FluentArgs;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::commands::AppState;
use crate::i18n::Texts;
use crate::tray::TrayItems;

/// How often the NAS is asked; the interface refreshes the open overview itself.
const INTERVAL: Duration = Duration::from_secs(60);
/// Update checks ask Synology's servers; a few times a day is plenty.
const UPDATE_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

fn rank(health: Health) -> u8 {
    match health {
        Health::Normal => 0,
        Health::Warning => 1,
        Health::Critical => 2,
    }
}

fn health_key(health: Health) -> &'static str {
    match health {
        Health::Normal => "health-normal",
        Health::Warning => "health-warning",
        Health::Critical => "health-critical",
    }
}

/// What the monitor remembers between two checks.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Status {
    reachable: bool,
    /// Volumes and drives that are not healthy, with their state.
    problems: Vec<(String, Health)>,
}

impl Status {
    fn from_overview(overview: &Overview, texts: &Texts) -> Self {
        let volumes = overview.volumes.iter().map(|volume| {
            let name = match volume.number {
                Some(number) => {
                    let mut args = FluentArgs::new();
                    args.set("number", number);
                    texts.get("volume-name", Some(&args))
                }
                None => volume.id.clone(),
            };
            (name, volume.health)
        });
        let drives = overview
            .drives
            .iter()
            .map(|drive| (drive.slot.clone(), drive.health));
        Self {
            reachable: true,
            problems: volumes
                .chain(drives)
                .filter(|(_, health)| *health != Health::Normal)
                .collect(),
        }
    }

    fn unreachable() -> Self {
        Self {
            reachable: false,
            problems: Vec::new(),
        }
    }

    fn worst(&self) -> Health {
        self.problems
            .iter()
            .map(|(_, health)| *health)
            .max_by_key(|health| rank(*health))
            .unwrap_or(Health::Normal)
    }
}

/// Notification texts for what changed since `before`.
fn changes(before: Option<&Status>, now: &Status, texts: &Texts) -> Vec<String> {
    let mut messages = Vec::new();
    match (before.map(|status| status.reachable), now.reachable) {
        (Some(true) | None, false) => messages.push(texts.get("notify-unreachable", None)),
        (Some(false), true) => messages.push(texts.get("notify-reachable", None)),
        _ => {}
    }
    for (part, health) in &now.problems {
        let known = before.and_then(|status| {
            status
                .problems
                .iter()
                .find(|(name, _)| name == part)
                .map(|(_, health)| *health)
        });
        if known.is_none_or(|known| rank(*health) > rank(known)) {
            let mut args = FluentArgs::new();
            args.set("part", part.clone());
            args.set("state", texts.get(health_key(*health), None));
            messages.push(texts.get("notify-problem", Some(&args)));
        }
    }
    let had_problems = before.is_some_and(|status| !status.problems.is_empty());
    if had_problems && now.reachable && now.problems.is_empty() {
        messages.push(texts.get("notify-healthy", None));
    }
    messages
}

fn tray_text(name: &str, status: &Status, texts: &Texts) -> String {
    let mut args = FluentArgs::new();
    args.set("name", name.to_owned());
    let state = if status.reachable {
        texts.get(health_key(status.worst()), None)
    } else {
        texts.get("tray-unreachable", None)
    };
    args.set("state", state);
    texts.get("tray-status", Some(&args))
}

fn notify(app: &AppHandle, title: &str, body: &str) {
    let state = app.state::<AppState>();
    if !state.preferences().notifications {
        return;
    }
    let _ = app.notification().builder().title(title).body(body).show();
}

/// What the monitor remembers about one signed-in NAS.
#[derive(Default)]
struct Watch {
    previous: Option<Status>,
    last_update_check: Option<Instant>,
    announced_dsm: Option<String>,
}

/// Runs for the lifetime of the app and watches every signed-in NAS.
pub async fn run(app: AppHandle) {
    let mut watches: HashMap<String, Watch> = HashMap::new();
    let mut shown: Option<String> = None;
    // A short first wait, so the tray shows the state soon after signing in.
    let mut wait = Duration::from_secs(5);

    loop {
        tokio::time::sleep(wait).await;
        wait = INTERVAL;
        let state = app.state::<AppState>();
        let targets = state.targets().await;
        watches.retain(|id, _| targets.iter().any(|target| &target.id == id));

        let mut lines = Vec::new();
        for target in &targets {
            let watch = watches.entry(target.id.clone()).or_default();
            let status = match target.client.overview(&target.session).await {
                Ok(overview) => Status::from_overview(&overview, &state.texts()),
                Err(Error::Network(_)) => Status::unreachable(),
                // An ended session is renewed by the interface; keep the last state.
                Err(_) => {
                    if let Some(previous) = &watch.previous {
                        lines.push(tray_text(&target.name, previous, &state.texts()));
                    }
                    continue;
                }
            };
            for message in changes(watch.previous.as_ref(), &status, &state.texts()) {
                notify(&app, &target.name, &message);
            }
            lines.push(tray_text(&target.name, &status, &state.texts()));
            let reachable = status.reachable;
            watch.previous = Some(status);

            let due = watch
                .last_update_check
                .is_none_or(|time| time.elapsed() >= UPDATE_INTERVAL);
            if reachable && due {
                watch.last_update_check = Some(Instant::now());
                let updates = target.client.updates(&target.session).await;
                if let Some(update) = updates.dsm
                    && watch.announced_dsm.as_deref() != Some(update.version.as_str())
                {
                    let mut args = FluentArgs::new();
                    args.set("version", update.version.clone());
                    let message = state.texts().get("notify-dsm-update", Some(&args));
                    notify(&app, &target.name, &message);
                    watch.announced_dsm = Some(update.version);
                }
            }
        }

        let text = if lines.is_empty() {
            state.texts().get("tray-not-connected", None)
        } else {
            lines.join(" · ")
        };
        if shown.as_deref() != Some(text.as_str())
            && let Some(tray) = app.try_state::<TrayItems>()
        {
            tray.set_status(&app, &text);
            shown = Some(text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(problems: &[(&str, Health)]) -> Status {
        Status {
            reachable: true,
            problems: problems
                .iter()
                .map(|(name, health)| ((*name).to_owned(), *health))
                .collect(),
        }
    }

    #[test]
    fn tells_about_new_and_worse_problems_once() {
        let texts = Texts::new(Some("en"));
        let first = status(&[("Drive 3", Health::Warning)]);
        assert_eq!(changes(None, &first, &texts), ["Drive 3: Warning"]);
        assert!(changes(Some(&first), &first, &texts).is_empty());
        let worse = status(&[("Drive 3", Health::Critical)]);
        assert_eq!(changes(Some(&first), &worse, &texts), ["Drive 3: Critical"]);
        let healthy = status(&[]);
        assert_eq!(
            changes(Some(&worse), &healthy, &texts),
            ["Everything is healthy again."]
        );
    }

    #[test]
    fn tells_when_the_nas_stops_and_starts_answering() {
        let texts = Texts::new(Some("en"));
        let up = status(&[]);
        let down = Status::unreachable();
        assert_eq!(
            changes(Some(&up), &down, &texts),
            ["The NAS does not answer."]
        );
        assert!(changes(Some(&down), &down, &texts).is_empty());
        assert_eq!(
            changes(Some(&down), &up, &texts),
            ["The NAS answers again."]
        );
    }

    #[test]
    fn tray_text_names_the_worst_state() {
        let texts = Texts::new(Some("en"));
        let mixed = status(&[("Drive 1", Health::Warning), ("Volume 1", Health::Critical)]);
        assert_eq!(tray_text("nas-home", &mixed, &texts), "nas-home: Critical");
        assert_eq!(
            tray_text("nas-home", &Status::unreachable(), &texts),
            "nas-home: not reachable"
        );
    }
}
