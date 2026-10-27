//! NAS the user connected to before. Nothing secret is stored here: passwords and
//! device tokens live in the system keychain (see `secrets`).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownDevice {
    /// `host:port`, lower case.
    pub id: String,
    pub name: String,
    pub model: String,
    pub host: String,
    pub port: u16,
    pub account: String,
    /// Fingerprint of a self-signed certificate the user trusted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    /// The password is kept in the keychain.
    #[serde(default)]
    pub remember_login: bool,
    /// MAC address seen by discovery, for Wake-on-LAN.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mac: Option<String>,
    /// SSH with a key of this app; the key itself is in the keychain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh: Option<SshLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshLink {
    pub port: u16,
    /// SHA-256 fingerprint of the host key the user trusted.
    pub host_key: String,
}

pub fn device_id(host: &str, port: u16) -> String {
    format!("{}:{port}", host.trim().to_lowercase())
}

/// The list of known NAS, kept as JSON in the app's config folder.
#[derive(Debug)]
pub struct DeviceStore {
    path: PathBuf,
    devices: Vec<KnownDevice>,
}

impl DeviceStore {
    /// Loads `devices.json` from `dir`; a missing or unreadable file starts an empty list.
    pub fn load(dir: &Path) -> Self {
        let path = dir.join("devices.json");
        let devices = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Self { path, devices }
    }

    pub fn all(&self) -> &[KnownDevice] {
        &self.devices
    }

    pub fn get(&self, id: &str) -> Option<&KnownDevice> {
        self.devices.iter().find(|device| device.id == id)
    }

    /// Adds or replaces a device and moves it to the top of the list.
    pub fn upsert(&mut self, device: KnownDevice) -> io::Result<()> {
        self.devices.retain(|known| known.id != device.id);
        self.devices.insert(0, device);
        self.save()
    }

    /// Notes the MAC address discovery saw for `host`; saves only when it changed.
    pub fn remember_mac(&mut self, host: &str, mac: &str) -> io::Result<()> {
        let mut changed = false;
        for device in self.devices.iter_mut().filter(|device| device.host == host) {
            if device.mac.as_deref() != Some(mac) {
                device.mac = Some(mac.to_owned());
                changed = true;
            }
        }
        if changed { self.save() } else { Ok(()) }
    }

    /// Replaces the SSH link of a saved device.
    pub fn set_ssh(&mut self, id: &str, ssh: Option<SshLink>) -> io::Result<()> {
        if let Some(device) = self.devices.iter_mut().find(|device| device.id == id) {
            device.ssh = ssh;
            self.save()?;
        }
        Ok(())
    }

    pub fn remove(&mut self, id: &str) -> io::Result<Option<KnownDevice>> {
        let index = self.devices.iter().position(|device| device.id == id);
        let removed = index.map(|index| self.devices.remove(index));
        self.save()?;
        Ok(removed)
    }

    /// Writes to a temporary file first, so a crash never leaves half a file.
    fn save(&self) -> io::Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)?;
        }
        let temporary = self.path.with_extension("json.tmp");
        fs::write(&temporary, serde_json::to_vec_pretty(&self.devices)?)?;
        fs::rename(&temporary, &self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(host: &str) -> KnownDevice {
        KnownDevice {
            id: device_id(host, 5001),
            name: host.into(),
            model: "DS923+".into(),
            host: host.into(),
            port: 5001,
            account: "admin".into(),
            fingerprint: None,
            remember_login: false,
            mac: None,
            ssh: None,
        }
    }

    #[test]
    fn keeps_devices_across_loads_newest_first() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = DeviceStore::load(dir.path());
        store.upsert(device("a")).unwrap();
        store.upsert(device("b")).unwrap();
        store.upsert(device("a")).unwrap();

        let reloaded = DeviceStore::load(dir.path());
        let ids: Vec<_> = reloaded.all().iter().map(|d| d.id.as_str()).collect();
        assert_eq!(ids, ["a:5001", "b:5001"]);

        let mut reloaded = reloaded;
        assert!(reloaded.remove("a:5001").unwrap().is_some());
        assert_eq!(DeviceStore::load(dir.path()).all().len(), 1);
    }

    #[test]
    fn remembers_macs_by_host() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = DeviceStore::load(dir.path());
        store.upsert(device("10.0.0.5")).unwrap();
        store.remember_mac("10.0.0.5", "00:11:32:aa:bb:cc").unwrap();
        store.remember_mac("10.0.0.9", "00:11:32:00:00:01").unwrap();
        let reloaded = DeviceStore::load(dir.path());
        assert_eq!(reloaded.all()[0].mac.as_deref(), Some("00:11:32:aa:bb:cc"));
    }

    #[test]
    fn keeps_the_ssh_link() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = DeviceStore::load(dir.path());
        store.upsert(device("a")).unwrap();
        let link = SshLink {
            port: 22,
            host_key: "SHA256:abc".into(),
        };
        store.set_ssh("a:5001", Some(link.clone())).unwrap();
        assert_eq!(DeviceStore::load(dir.path()).all()[0].ssh, Some(link));
    }

    #[test]
    fn ids_ignore_case_and_spaces() {
        assert_eq!(device_id(" NAS.local ", 5001), "nas.local:5001");
    }
}
