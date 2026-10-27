//! What DSM's web interface does not tell: kernel, io_uring, network drivers and
//! memory modules. One script collects everything in a single round trip; the
//! memory modules need root and come from a second, optional call.

use serde::Serialize;

use crate::{Connection, Error};

/// Prints one section per topic, each introduced by an `@@name` line.
const SCRIPT: &str = r#"echo @@version; cat /etc.defaults/VERSION 2>/dev/null
echo @@uname; uname -r; uname -m
echo @@iouring; grep -c io_uring_setup /proc/kallsyms 2>/dev/null; cat /proc/sys/kernel/io_uring_disabled 2>/dev/null
echo @@nics; for n in /sys/class/net/*; do [ -e "$n/device/driver" ] || continue; printf '%s %s %s\n' "${n##*/}" "$(basename "$(readlink "$n/device/driver")")" "$(cat "$n/speed" 2>/dev/null)"; done
echo @@docker; docker --version 2>/dev/null || /usr/local/bin/docker --version 2>/dev/null
echo @@end"#;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    /// "7.2.2"
    pub dsm_version: Option<String>,
    pub dsm_build: Option<String>,
    /// The "Update n" on top of the build.
    pub dsm_update: Option<u32>,
    /// "4.4.302+"
    pub kernel: String,
    /// "x86_64", "aarch64"
    pub machine: String,
    pub io_uring: IoUring,
    pub network: Vec<NetworkDriver>,
    /// "Docker version 24.0.2, build 610b8d0"
    pub docker: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IoUring {
    Available,
    /// Built in, but switched off with `kernel.io_uring_disabled`.
    Disabled,
    /// Not in this kernel; io_uring arrived with Linux 5.1.
    #[default]
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDriver {
    pub interface: String,
    pub driver: String,
    /// Link speed in Mbit/s, when the link is up.
    pub speed: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryModule {
    pub slot: String,
    pub size_mb: Option<u64>,
    /// "DDR4"
    pub kind: Option<String>,
    /// MT/s
    pub speed: Option<u32>,
    pub manufacturer: Option<String>,
    pub part_number: Option<String>,
}

impl Connection {
    pub async fn diagnostics(&self) -> Result<Diagnostics, Error> {
        Ok(parse(&self.run(SCRIPT).await?.stdout))
    }

    /// The installed memory modules, read with `dmidecode` as root.
    pub async fn memory_modules(&self, password: &str) -> Result<Vec<MemoryModule>, Error> {
        let output = self.run_sudo("dmidecode -t 17", password).await?;
        if output.sudo_refused() {
            return Err(Error::AuthFailed);
        }
        if !output.success() {
            return Err(Error::Protocol(output.stderr.trim().to_owned()));
        }
        Ok(parse_memory(&output.stdout))
    }
}

fn sections(text: &str) -> impl Iterator<Item = (&str, Vec<&str>)> {
    text.split("@@").skip(1).filter_map(|block| {
        let mut lines = block.lines();
        let name = lines.next()?.trim();
        Some((
            name,
            lines
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .collect(),
        ))
    })
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').to_owned()
}

pub(crate) fn parse(text: &str) -> Diagnostics {
    let mut result = Diagnostics::default();
    for (name, lines) in sections(text) {
        match name {
            "version" => {
                for (key, value) in lines.iter().filter_map(|line| line.split_once('=')) {
                    match key {
                        "productversion" => result.dsm_version = Some(unquote(value)),
                        "buildnumber" => result.dsm_build = Some(unquote(value)),
                        "smallfixnumber" => {
                            result.dsm_update = unquote(value).parse().ok().filter(|n| *n > 0);
                        }
                        _ => {}
                    }
                }
            }
            "uname" => {
                result.kernel = lines.first().copied().unwrap_or_default().to_owned();
                result.machine = lines.get(1).copied().unwrap_or_default().to_owned();
            }
            "iouring" => {
                let symbols: u32 = lines.first().and_then(|n| n.parse().ok()).unwrap_or(0);
                let disabled: u32 = lines.get(1).and_then(|n| n.parse().ok()).unwrap_or(0);
                result.io_uring = match (symbols, disabled) {
                    (0, _) => IoUring::Missing,
                    // 1 allows only privileged processes, 2 nobody.
                    (_, 2) => IoUring::Disabled,
                    _ => IoUring::Available,
                };
            }
            "nics" => {
                result.network = lines
                    .iter()
                    .filter_map(|line| {
                        let mut parts = line.split_whitespace();
                        Some(NetworkDriver {
                            interface: parts.next()?.to_owned(),
                            driver: parts.next()?.to_owned(),
                            speed: parts.next().and_then(|n| n.parse().ok()),
                        })
                    })
                    .collect();
            }
            "docker" => result.docker = lines.first().map(|line| (*line).to_owned()),
            _ => {}
        }
    }
    result
}

fn known(value: &str) -> Option<String> {
    let value = value.trim();
    let unknown = [
        "",
        "Unknown",
        "Not Specified",
        "NO DIMM",
        "Undefined",
        "<OUT OF SPEC>",
    ];
    (!unknown.contains(&value)).then(|| value.to_owned())
}

/// The "Memory Device" records of `dmidecode -t 17`; empty slots are left out.
pub(crate) fn parse_memory(text: &str) -> Vec<MemoryModule> {
    text.split("\n\n")
        .filter(|record| record.contains("Memory Device"))
        .filter_map(|record| {
            let field = |name: &str| {
                record.lines().find_map(|line| {
                    let (key, value) = line.trim().split_once(':')?;
                    (key == name).then(|| value.trim())
                })
            };
            let size = field("Size")?;
            let size_mb = match size.split_whitespace().collect::<Vec<_>>()[..] {
                [n, "MB"] => n.parse().ok(),
                [n, "GB"] => n.parse::<u64>().ok().map(|n| n * 1024),
                _ => return None,
            };
            let speed = field("Configured Memory Speed")
                .or_else(|| field("Speed"))
                .and_then(|speed| speed.split_whitespace().next()?.parse().ok());
            Some(MemoryModule {
                slot: field("Locator").and_then(known).unwrap_or_default(),
                size_mb,
                kind: field("Type").and_then(known),
                speed,
                manufacturer: field("Manufacturer").and_then(known),
                part_number: field("Part Number").and_then(known),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_script_output() {
        let text = "@@version\nmajorversion=\"7\"\nproductversion=\"7.2.2\"\nbuildnumber=\"72806\"\nsmallfixnumber=\"3\"\n\
            @@uname\n4.4.302+\nx86_64\n@@iouring\n0\n@@nics\neth0 r8169 1000\neth1 r8169 \n@@docker\n@@end\n";
        let result = parse(text);
        assert_eq!(result.dsm_version.as_deref(), Some("7.2.2"));
        assert_eq!(result.dsm_build.as_deref(), Some("72806"));
        assert_eq!(result.dsm_update, Some(3));
        assert_eq!(result.kernel, "4.4.302+");
        assert_eq!(result.machine, "x86_64");
        assert_eq!(result.io_uring, IoUring::Missing);
        assert_eq!(result.network.len(), 2);
        assert_eq!(result.network[0].speed, Some(1000));
        assert_eq!(result.network[1].speed, None);
        assert_eq!(result.docker, None);
    }

    #[test]
    fn io_uring_states() {
        let state = |lines: &str| parse(&format!("@@iouring\n{lines}@@end")).io_uring;
        assert_eq!(state("4\n"), IoUring::Available);
        assert_eq!(state("4\n0\n"), IoUring::Available);
        assert_eq!(state("4\n2\n"), IoUring::Disabled);
        assert_eq!(state(""), IoUring::Missing);
    }

    #[test]
    fn reads_memory_modules() {
        let text = "# dmidecode 3.2\n\nHandle 0x0011, DMI type 17, 40 bytes\nMemory Device\n\
            \tSize: 4096 MB\n\tLocator: ChannelA-DIMM0\n\tType: DDR4\n\tSpeed: 2667 MT/s\n\
            \tManufacturer: Samsung\n\tPart Number: M471A5244CB0-CTD    \n\
            \tConfigured Memory Speed: 2400 MT/s\n\n\
            Handle 0x0013, DMI type 17, 40 bytes\nMemory Device\n\tSize: No Module Installed\n\
            \tLocator: ChannelB-DIMM0\n\n\
            Handle 0x0014, DMI type 17, 40 bytes\nMemory Device\n\tSize: 16 GB\n\tLocator: DIMM 1\n\
            \tType: RAM\n\tSpeed: Unknown\n\tManufacturer: QEMU\n\tPart Number: Not Specified\n";
        let modules = parse_memory(text);
        assert_eq!(modules.len(), 2);
        assert_eq!(modules[0].size_mb, Some(4096));
        assert_eq!(modules[0].speed, Some(2400));
        assert_eq!(modules[0].part_number.as_deref(), Some("M471A5244CB0-CTD"));
        assert_eq!(modules[1].size_mb, Some(16384));
        assert_eq!(modules[1].speed, None);
        assert_eq!(modules[1].part_number, None);
    }
}
