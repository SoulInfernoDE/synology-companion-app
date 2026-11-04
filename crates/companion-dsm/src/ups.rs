//! The UPS that protects the NAS, from `SYNO.Core.ExternalDevice.UPS`.
//!
//! DSM reports a UPS on USB, one shared by a network UPS server (another NAS or a
//! NUT server) and one over SNMP through the same call. Charge, runtime and load
//! are only meaningful while the UPS answers, so they stay empty otherwise.

use serde::Serialize;
use serde_json::Value;

use crate::overview::{Health, number, text};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UpsStatus {
    /// On mains power.
    Online,
    OnBattery,
    LowBattery,
    Off,
    /// Support is on, but DSM gets no answer from the UPS or its server.
    Unknown,
}

impl UpsStatus {
    /// DSM's values are the names of its interface texts, e.g. "usb_ups_status_online".
    fn from_api(status: Option<&str>) -> Self {
        match status.and_then(|status| status.strip_prefix("usb_ups_status_")) {
            Some("online") => Self::Online,
            Some("battery") => Self::OnBattery,
            Some("low_batt") => Self::LowBattery,
            Some("off") => Self::Off,
            _ => Self::Unknown,
        }
    }

    fn answers(self) -> bool {
        matches!(self, Self::Online | Self::OnBattery | Self::LowBattery)
    }

    pub fn health(self) -> Health {
        match self {
            Self::Online => Health::Normal,
            Self::LowBattery => Health::Critical,
            Self::OnBattery | Self::Off | Self::Unknown => Health::Warning,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ups {
    pub status: UpsStatus,
    pub health: Health,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    /// Address of the network UPS server when the UPS is not attached to this NAS.
    pub server: Option<String>,
    pub charge_percent: Option<f64>,
    /// Estimated time on battery.
    pub runtime_seconds: Option<u64>,
    pub load_percent: Option<f64>,
}

impl Ups {
    /// `None` when UPS support is off in DSM.
    pub(crate) fn from_api(ups: &Value) -> Option<Self> {
        if ups["enable"].as_bool() != Some(true) {
            return None;
        }
        let status = UpsStatus::from_api(text(ups, &["status"]));
        let reading =
            |keys: &[&str]| number(ups, keys).filter(|value| status.answers() && *value >= 0.0);
        let server = (text(ups, &["mode"]) == Some("SLAVE"))
            .then(|| text(ups, &["net_server_ip"]))
            .flatten()
            .map(str::to_owned);
        Some(Self {
            status,
            health: status.health(),
            manufacturer: text(ups, &["manufacture", "manufacturer"]).map(str::to_owned),
            model: text(ups, &["model"]).map(str::to_owned),
            server,
            charge_percent: reading(&["charge"]).map(|charge| charge.min(100.0)),
            runtime_seconds: reading(&["runtime"])
                .filter(|seconds| *seconds > 0.0)
                .map(|seconds| seconds as u64),
            load_percent: reading(&["load"]),
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// Answer of a Virtual DSM 7.2.2 without a UPS.
    #[test]
    fn support_off_means_no_ups() {
        let path = format!(
            "{}/tests/fixtures/dsm-7.2.2-virtualdsm/ups-get.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let answer: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(Ups::from_api(&answer), None);
    }

    #[test]
    fn reads_a_usb_ups() {
        let ups = Ups::from_api(&json!({
            "enable": true, "mode": "MASTER", "status": "usb_ups_status_online",
            "manufacture": "APC", "model": "Back-UPS ES 700G", "charge": 100,
            "runtime": "1680", "load": 18, "net_server_ip": "", "usb_ups_connect": true
        }))
        .unwrap();
        assert_eq!(ups.status, UpsStatus::Online);
        assert_eq!(ups.health, Health::Normal);
        assert_eq!(ups.manufacturer.as_deref(), Some("APC"));
        assert_eq!(ups.charge_percent, Some(100.0));
        assert_eq!(ups.runtime_seconds, Some(1680));
        assert_eq!(ups.load_percent, Some(18.0));
        assert_eq!(ups.server, None);
    }

    #[test]
    fn reads_a_network_ups_on_battery() {
        let ups = Ups::from_api(&json!({
            "enable": true, "mode": "SLAVE", "status": "usb_ups_status_battery",
            "manufacture": "", "model": "", "charge": 64, "runtime": 900,
            "net_server_ip": "ups-server.example"
        }))
        .unwrap();
        assert_eq!(ups.status, UpsStatus::OnBattery);
        assert_eq!(ups.health, Health::Warning);
        assert_eq!(ups.model, None);
        assert_eq!(ups.server.as_deref(), Some("ups-server.example"));
        assert_eq!(ups.load_percent, None);
    }

    #[test]
    fn no_readings_without_an_answer() {
        let ups = Ups::from_api(&json!({
            "enable": true, "mode": "SLAVE", "status": "usb_ups_status_unknown",
            "charge": 0, "runtime": 0, "net_server_ip": "ups-server.example"
        }))
        .unwrap();
        assert_eq!(ups.status, UpsStatus::Unknown);
        assert_eq!(ups.charge_percent, None);
        assert_eq!(ups.runtime_seconds, None);
        assert_eq!(
            UpsStatus::from_api(Some("usb_ups_status_low_batt")).health(),
            Health::Critical
        );
    }
}
