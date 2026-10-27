//! Download Station: its tasks, adding links and pausing, resuming or removing
//! tasks, through `SYNO.DownloadStation.Task`.

use serde::Serialize;
use serde_json::Value;

use crate::{Client, Error, Session};

const API: &str = "SYNO.DownloadStation.Task";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub id: String,
    pub title: String,
    /// "waiting", "downloading", "paused", "finishing", "finished",
    /// "hash_checking", "seeding", "extracting", "error", …
    pub status: String,
    /// Why a task stopped with an error, as Download Station names it.
    pub error: Option<String>,
    /// Bytes.
    pub size: u64,
    pub downloaded: u64,
    /// Bytes per second.
    pub speed: u64,
    pub destination: Option<String>,
}

fn number(value: &Value) -> u64 {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
        .unwrap_or(0)
}

pub(crate) fn parse(list: &Value) -> Vec<Download> {
    list["tasks"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(|task| {
            let transfer = &task["additional"]["transfer"];
            Some(Download {
                id: task["id"].as_str()?.to_owned(),
                title: task["title"].as_str().unwrap_or_default().to_owned(),
                status: task["status"].as_str().unwrap_or_default().to_owned(),
                error: task["status_extra"]["error_detail"]
                    .as_str()
                    .filter(|detail| !detail.is_empty())
                    .map(str::to_owned),
                size: number(&task["size"]),
                downloaded: number(&transfer["size_downloaded"]),
                speed: number(&transfer["speed_download"]),
                destination: task["additional"]["detail"]["destination"]
                    .as_str()
                    .filter(|destination| !destination.is_empty())
                    .map(str::to_owned),
            })
        })
        .collect()
}

/// What can be done with a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DownloadAction {
    Pause,
    Resume,
    Delete,
}

impl Client {
    pub async fn downloads(&self, session: &Session) -> Result<Vec<Download>, Error> {
        let list = self
            .call(
                session,
                API,
                "list",
                1,
                &[("additional", "detail,transfer"), ("limit", "-1")],
            )
            .await?;
        Ok(parse(&list))
    }

    /// Adds a download: an HTTP, HTTPS or FTP link or a magnet link.
    pub async fn add_download(&self, session: &Session, uri: &str) -> Result<(), Error> {
        self.call(session, API, "create", 1, &[("uri", uri)])
            .await
            .map(drop)
    }

    pub async fn download_action(
        &self,
        session: &Session,
        id: &str,
        action: DownloadAction,
    ) -> Result<(), Error> {
        let (method, extra): (&str, &[(&str, &str)]) = match action {
            DownloadAction::Pause => ("pause", &[]),
            DownloadAction::Resume => ("resume", &[]),
            // Keeps what was downloaded so far, as Download Station's own button does.
            DownloadAction::Delete => ("delete", &[("force_complete", "false")]),
        };
        let mut form = vec![("id", id)];
        form.extend_from_slice(extra);
        self.call(session, API, method, 1, &form).await.map(drop)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn reads_tasks() {
        let list = json!({ "offset": 0, "total": 2, "tasks": [
            { "id": "dbid_1", "title": "debian.iso", "status": "downloading", "size": 657457152,
              "additional": { "transfer": { "size_downloaded": 1000, "speed_download": 250 },
                              "detail": { "destination": "downloads" } } },
            { "id": "dbid_2", "title": "broken", "status": "error", "size": "0",
              "status_extra": { "error_detail": "broken_link" } }
        ]});
        let downloads = parse(&list);
        assert_eq!(downloads.len(), 2);
        assert_eq!(downloads[0].downloaded, 1000);
        assert_eq!(downloads[0].speed, 250);
        assert_eq!(downloads[0].destination.as_deref(), Some("downloads"));
        assert_eq!(downloads[1].error.as_deref(), Some("broken_link"));
        assert_eq!(downloads[1].size, 0);
    }
}
