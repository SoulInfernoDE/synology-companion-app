//! A stand-in for the DSM Web API, to try Syno Companion without a NAS:
//!
//! ```text
//! cargo run -p companion-mockdsm -- [--port 15001] [--otp] [--unconfigured]
//! ```
//!
//! It serves HTTPS with a fresh self-signed certificate, like DSM does out of the
//! box, and answers with the answers recorded from a Virtual DSM 7.4.1 plus sample
//! data for what a virtual NAS lacks (drives, temperature, updates). Sign in with
//! account `demo` and password `demo`; with `--otp` that account needs the code
//! `123456`, as an account with two-step verification would. With
//! `--unconfigured` it behaves like a NAS whose welcome wizard has not run yet.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const ACCOUNT: &str = "demo";
const PASSWORD: &str = "demo";
const OTP: &str = "123456";
const SID: &str = "mock-session";
const DEVICE_TOKEN: &str = "mock-device";
const SETUP_COOKIE: &str = "mock-setup";

const SYSTEM: &str =
    include_str!("../../companion-dsm/tests/fixtures/dsm-7.4.1-virtualdsm/system-info.json");
const UTILIZATION: &str =
    include_str!("../../companion-dsm/tests/fixtures/dsm-7.4.1-virtualdsm/utilization.json");

struct Options {
    port: u16,
    otp: bool,
    unconfigured: bool,
}

/// What changes while the mock runs.
struct World {
    configured: bool,
    /// Accounts created through the first-time setup, with their passwords.
    accounts: HashMap<String, String>,
    /// DSM's SSH service; the mock only answers the setting, it runs no SSH server.
    ssh: bool,
    /// Package sources as (name, address), and Package Center's trust level.
    feeds: Vec<(String, String)>,
    trust_level: u8,
    /// Download Station tasks as (id, title, status).
    downloads: Vec<(String, String, String)>,
}

fn options() -> Options {
    let mut options = Options {
        port: 15001,
        otp: false,
        unconfigured: false,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--port" => {
                options.port = args
                    .next()
                    .and_then(|port| port.parse().ok())
                    .expect("--port needs a number");
            }
            "--otp" => options.otp = true,
            "--unconfigured" => options.unconfigured = true,
            other => panic!("unknown option {other}; use --port N, --otp, --unconfigured"),
        }
    }
    options
}

fn success(data: Value) -> Value {
    json!({ "success": true, "data": data })
}

fn failure(code: i64) -> Value {
    json!({ "success": false, "error": { "code": code } })
}

fn api_list() -> Value {
    let apis = [
        "SYNO.API.Auth",
        "SYNO.Core.Desktop.SessionData",
        "SYNO.Core.FileServ.SMB",
        "SYNO.Core.Group.Member",
        "SYNO.Core.Hardware.BeepControl",
        "SYNO.Core.Network",
        "SYNO.Core.Network.Ethernet",
        "SYNO.Core.Package",
        "SYNO.Core.Package.Feed",
        "SYNO.Core.Package.Server",
        "SYNO.Core.Package.Setting",
        "SYNO.Core.QuickStart.Info",
        "SYNO.Core.Share",
        "SYNO.Core.System",
        "SYNO.Core.System.Utilization",
        "SYNO.Core.Terminal",
        "SYNO.Core.Upgrade.Server",
        "SYNO.Core.User",
        "SYNO.Core.User.Home",
        "SYNO.Core.User.PasswordPolicy",
        "SYNO.Core.User.UsernamePolicy",
        "SYNO.DownloadStation.Task",
        "SYNO.Entry.Request",
        "SYNO.Storage.CGI.Storage",
    ];
    let entries: serde_json::Map<String, Value> = apis
        .iter()
        .map(|api| {
            (
                (*api).to_owned(),
                // Download Station keeps its own CGI path, as on a real NAS.
                json!({
                    "path": if api.starts_with("SYNO.DownloadStation") {
                        "DownloadStation/task.cgi"
                    } else {
                        "entry.cgi"
                    },
                    "minVersion": 1,
                    "maxVersion": 7,
                }),
            )
        })
        .collect();
    success(Value::Object(entries))
}

fn system() -> Value {
    let mut system: Value = serde_json::from_str(SYSTEM).expect("fixture is JSON");
    system["model"] = json!("DS923+");
    system["cpu_vendor"] = json!("AMD");
    system["cpu_family"] = json!("Ryzen");
    system["cpu_series"] = json!("R1600");
    system["cpu_cores"] = json!("2");
    system["cpu_clock_speed"] = json!(2600);
    system["sys_temp"] = json!(41);
    system
}

fn storage() -> Value {
    let tb = 1_000_000_000_000_u64;
    let disk = |id: &str, slot: &str, model: &str, temp: u32, status: &str, ssd: bool| {
        json!({ "id": id, "longName": slot, "model": model, "vendor": "", "temp": temp,
                "size_total": (12 * tb).to_string(), "overview_status": status, "isSsd": ssd,
                "diskType": if ssd { "M.2 NVMe" } else { "SATA" }, "compatibility": "not_support" })
    };
    json!({
        "storagePools": [{ "id": "reuse_1", "device_type": "shr_with_1_disk_protect" }],
        "volumes": [
            { "id": "volume_1", "vol_path": "/volume1", "pool_path": "reuse_1", "status": "normal",
              "fs_type": "btrfs", "size": { "total": (22 * tb).to_string(), "used": (13 * tb).to_string() } },
            { "id": "volume_2", "vol_path": "/volume2", "device_type": "basic", "status": "normal",
              "fs_type": "btrfs", "size": { "total": "980000000000", "used": "210000000000" } }
        ],
        "disks": [
            disk("sata1", "Drive 1", "WD Red Plus 12TB", 36, "normal", false),
            disk("sata2", "Drive 2", "WD Red Plus 12TB", 37, "normal", false),
            disk("sata3", "Drive 3", "Seagate IronWolf 12TB", 39, "warning", false),
            disk("nvme0n1", "M.2 Drive 1", "Samsung 990 Pro 1TB", 44, "normal", true)
        ]
    })
}

fn installed_packages() -> Value {
    json!({ "packages": [
        { "id": "SMBService", "name": "SMB Service", "version": "4.15.13-0871" },
        { "id": "FileStation", "name": "File Station", "version": "1.4.2-1575" }
    ]})
}

fn catalogue(feeds: bool) -> Value {
    let mut packages = vec![
        json!({ "id": "SMBService", "dname": "SMB Service", "version": "4.15.13-2503",
                "source": "syno", "desc": "Windows file sharing." }),
        json!({ "id": "FileStation", "dname": "File Station", "version": "1.4.2-1575",
                "source": "syno", "desc": "Manage files in the browser." }),
        json!({ "id": "TextEditor", "dname": "Text Editor", "version": "1.2.5-0254",
                "source": "syno", "desc": "Edit text files on the NAS." }),
    ];
    if feeds {
        packages.push(json!({ "id": "synocli-file", "dname": "SynoCli File Tools",
            "version": "4.0.3-32", "source": "others", "distributor": "SynoCommunity",
            "desc": "Command-line tools such as nano, less and tree.",
            "link": "https://packages.synocommunity.com/synocli-file.spk",
            "md5": "708543379fa661d5bfabb5d179ea7d42" }));
    }
    json!({ "packages": packages })
}

/// The answer to one Web API call.
fn answer(
    params: &HashMap<String, String>,
    cookie: bool,
    options: &Options,
    world: &mut World,
) -> Value {
    let param = |name: &str| params.get(name).map(String::as_str).unwrap_or_default();
    let api = param("api");
    let method = param("method");
    let signed_in = param("_sid") == SID;
    let setup_session = !world.configured && cookie;

    match (api, method) {
        ("SYNO.API.Info", "query") => api_list(),
        ("SYNO.API.Auth", "login") => {
            let account = param("account");
            let password = param("passwd");
            let known = (account == ACCOUNT && password == PASSWORD)
                || world
                    .accounts
                    .get(account)
                    .is_some_and(|known| known == password);
            if !known {
                failure(400)
            } else if options.otp && account == ACCOUNT && params.get("device_id").is_none() {
                match params.get("otp_code").map(String::as_str) {
                    None => failure(403),
                    Some(OTP) => success(json!({ "sid": SID, "did": DEVICE_TOKEN })),
                    Some(_) => failure(404),
                }
            } else {
                success(json!({ "sid": SID }))
            }
        }
        ("SYNO.API.Auth", "logout") => success(json!({})),
        ("SYNO.Core.QuickStart.Info", "check_permission") if setup_session => success(json!({})),
        ("SYNO.Entry.Request", "request") if setup_session || signed_in => {
            let calls: Vec<Value> = serde_json::from_str(param("compound")).unwrap_or_default();
            let mut results = Vec::new();
            for call in &calls {
                let api = call["api"].as_str().unwrap_or_default();
                let method = call["method"].as_str().unwrap_or_default();
                let data = match (api, method) {
                    ("SYNO.Core.User", "create") => {
                        let name = call["name"].as_str().unwrap_or_default().to_owned();
                        let password = call["password"].as_str().unwrap_or_default().to_owned();
                        world.accounts.insert(name, password);
                        json!({})
                    }
                    ("SYNO.Core.User", "set") if call["expired"] == "now" => {
                        world.configured = true;
                        json!({})
                    }
                    ("SYNO.Core.User.PasswordPolicy", "get") => json!({ "strong_password": {
                        "min_length_enable": true, "min_length": 8, "mixed_case": true,
                        "included_numeric_char": true, "exclude_username": true } }),
                    ("SYNO.Core.User.UsernamePolicy", "list") => json!(["admin", "guest", "root"]),
                    ("SYNO.Core.Region.NTP", "listzone") => json!({ "zonedata": [
                        { "value": "London", "display": "(GMT) Dublin, Edinburgh, Lisbon, London", "offset": 0 },
                        { "value": "Amsterdam", "display": "(GMT+01:00) Amsterdam, Berlin, Rome, Stockholm, Vienna", "offset": 3600 }
                    ] }),
                    ("SYNO.Core.Network", "get") => json!({ "server_name": "mock-dsm" }),
                    _ => json!({}),
                };
                results
                    .push(json!({ "api": api, "method": method, "success": true, "data": data }));
            }
            success(json!({ "has_fail": false, "result": results }))
        }
        _ if !signed_in => failure(119),
        ("SYNO.Core.System", "info") => success(system()),
        ("SYNO.Core.System.Utilization", "get") => {
            success(serde_json::from_str(UTILIZATION).expect("fixture is JSON"))
        }
        ("SYNO.Storage.CGI.Storage", "load_info") => success(storage()),
        ("SYNO.Core.Network", "get") => success(json!({ "server_name": "mock-dsm" })),
        ("SYNO.Core.Network.Ethernet", "list") => success(json!([
            { "ifname": "eth0", "status": "connected", "speed": 1000, "max_supported_speed": 2500 },
            { "ifname": "eth1", "status": "disconnected", "speed": -1, "max_supported_speed": 2500 }
        ])),
        ("SYNO.Core.Upgrade.Server", "check") => success(json!({ "update": {
            "available": true, "reboot": "now", "version": "DSM 7.4.1-90080 Update 1" } })),
        ("SYNO.Core.Package", "list") => success(installed_packages()),
        ("SYNO.Core.Package.Server", "list") => success(catalogue(!world.feeds.is_empty())),
        ("SYNO.Core.Package.Feed", "list") => success(json!({ "items": world.feeds.iter()
            .map(|(name, feed)| json!({ "name": name, "feed": feed })).collect::<Vec<_>>() })),
        ("SYNO.Core.Package.Feed", "add") => {
            // DSM sends the entry as JSON text inside a JSON string.
            let entry: Option<Value> = serde_json::from_str::<String>(param("list"))
                .ok()
                .and_then(|text| serde_json::from_str(&text).ok());
            match entry {
                Some(entry)
                    if entry["feed"]
                        .as_str()
                        .is_some_and(|f| f.starts_with("https://")) =>
                {
                    world.feeds.push((
                        entry["name"].as_str().unwrap_or_default().to_owned(),
                        entry["feed"].as_str().unwrap_or_default().to_owned(),
                    ));
                    success(json!({}))
                }
                _ => failure(4590),
            }
        }
        ("SYNO.Core.Package.Feed", "delete") => {
            let urls: Vec<String> = serde_json::from_str::<String>(param("list"))
                .ok()
                .and_then(|text| serde_json::from_str(&text).ok())
                .unwrap_or_default();
            world.feeds.retain(|(_, feed)| !urls.contains(feed));
            success(json!({}))
        }
        ("SYNO.Core.Package.Setting", "get") => {
            success(json!({ "trust_level": world.trust_level, "enable_autoupdate": false }))
        }
        ("SYNO.Core.Package.Setting", "set") => {
            if let Ok(level) = param("trust_level").parse() {
                world.trust_level = level;
            }
            success(json!({}))
        }
        ("SYNO.Core.Share", "list") => success(json!({ "shares": [
            { "name": "homes", "desc": "homes contains all users' home", "hidden": false, "encryption": 0 },
            { "name": "media", "desc": "Photos, music and videos", "hidden": false, "encryption": 0 }
        ] })),
        ("SYNO.Core.FileServ.SMB", "get") => success(json!({ "enable_samba": true })),
        ("SYNO.Core.Terminal", "get") => {
            success(json!({ "enable_ssh": world.ssh, "enable_telnet": false, "ssh_port": 22 }))
        }
        ("SYNO.Core.Terminal", "set") => {
            world.ssh = param("enable_ssh") == "true";
            success(json!({}))
        }
        ("SYNO.Core.User.Home", "get") => success(json!({ "enable": true })),
        ("SYNO.DownloadStation.Task", "list") => success(json!({
            "total": world.downloads.len(),
            "tasks": world.downloads.iter().map(|(id, title, status)| json!({
                "id": id, "title": title, "status": status, "size": 700_000_000u64,
                "additional": {
                    "transfer": {
                        "size_downloaded": if status == "finished" { 700_000_000u64 } else { 280_000_000 },
                        "speed_download": if status == "downloading" { 4_200_000u64 } else { 0 },
                    },
                    "detail": { "destination": "downloads" },
                },
            })).collect::<Vec<_>>(),
        })),
        ("SYNO.DownloadStation.Task", "create") => {
            let uri = param("uri");
            let title = uri.rsplit('/').next().unwrap_or(uri).to_owned();
            let id = format!("dbid_{}", world.downloads.len() + 10);
            world.downloads.push((id, title, "waiting".into()));
            success(json!({}))
        }
        ("SYNO.DownloadStation.Task", "pause" | "resume" | "delete") => {
            let id = param("id").to_owned();
            match method {
                "delete" => world.downloads.retain(|(known, ..)| *known != id),
                _ => {
                    let status = if method == "pause" {
                        "paused"
                    } else {
                        "downloading"
                    };
                    for task in world
                        .downloads
                        .iter_mut()
                        .filter(|(known, ..)| *known == id)
                    {
                        task.2 = status.into();
                    }
                }
            }
            success(json!({}))
        }
        ("SYNO.Core.System", "reboot" | "shutdown")
        | ("SYNO.Core.Hardware.BeepControl", "stop_beep") => success(json!({})),
        _ => failure(103),
    }
}

fn parse_form(text: &str) -> HashMap<String, String> {
    url::form_urlencoded::parse(text.as_bytes())
        .into_owned()
        .collect()
}

fn respond(stream: &mut impl Write, status: &str, extra: &str, body: &str) {
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{extra}Connection: close\r\n\r\n{body}",
        body.len()
    );
}

fn serve(
    stream: TcpStream,
    config: Arc<rustls::ServerConfig>,
    options: &Options,
    world: &Mutex<World>,
) {
    let Ok(connection) = rustls::ServerConnection::new(config) else {
        return;
    };
    let mut tls = rustls::StreamOwned::new(connection, stream);
    let mut reader = BufReader::new(&mut tls);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return; // The client refused the certificate, as it should the first time.
    }
    let mut length = 0;
    let mut cookie = false;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).is_err() || header.trim().is_empty() {
            break;
        }
        let lower = header.to_ascii_lowercase();
        if let Some(value) = lower.strip_prefix("content-length:") {
            length = value.trim().parse().unwrap_or(0);
        }
        if lower.starts_with("cookie:") && lower.contains(SETUP_COOKIE) {
            cookie = true;
        }
    }
    let mut body = vec![0; length];
    if reader.read_exact(&mut body).is_err() {
        return;
    }

    let target = request_line.split_whitespace().nth(1).unwrap_or("/");
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let mut world = world
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if path == "/" {
        // A NAS that is not set up hands out its setup session to every visitor.
        let cookie = if world.configured {
            String::new()
        } else {
            format!("Set-Cookie: id={SETUP_COOKIE}; path=/\r\n")
        };
        respond(&mut tls, "200 OK", &cookie, "{}");
        return;
    }
    let mut params = parse_form(query);
    params.extend(parse_form(&String::from_utf8_lossy(&body)));
    let answer = answer(&params, cookie, options, &mut world);
    respond(&mut tls, "200 OK", "", &answer.to_string());
    tls.conn.send_close_notify();
    let _ = tls.flush();
}

fn main() {
    let options = options();
    let certified = rcgen::generate_simple_self_signed(vec!["localhost".to_owned()])
        .expect("generate a certificate");
    let certificate = certified.cert.der().clone();
    let fingerprint = Sha256::digest(&certificate)
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(":");
    let key = rustls::pki_types::PrivateKeyDer::Pkcs8(certified.signing_key.serialize_der().into());
    let config = Arc::new(
        rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .expect("TLS versions")
        .with_no_client_auth()
        .with_single_cert(vec![certificate], key)
        .expect("certificate and key"),
    );
    let listener = TcpListener::bind(("127.0.0.1", options.port)).expect("bind the port");
    println!("Mock DSM on https://127.0.0.1:{}", options.port);
    println!("Certificate fingerprint: {fingerprint}");
    println!(
        "Sign in with {ACCOUNT} / {PASSWORD}{}{}",
        if options.otp { ", code 123456" } else { "" },
        if options.unconfigured {
            "; not set up yet"
        } else {
            ""
        }
    );

    let world = Arc::new(Mutex::new(World {
        configured: !options.unconfigured,
        accounts: HashMap::new(),
        ssh: false,
        feeds: Vec::new(),
        trust_level: 0,
        downloads: vec![
            (
                "dbid_1".into(),
                "debian-13.1.0-amd64-netinst.iso".into(),
                "downloading".into(),
            ),
            (
                "dbid_2".into(),
                "big-buck-bunny.mp4".into(),
                "finished".into(),
            ),
        ],
    }));
    let options = Arc::new(options);
    for stream in listener.incoming().flatten() {
        let (config, options, world) = (config.clone(), options.clone(), world.clone());
        std::thread::spawn(move || serve(stream, config, &options, &world));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    }

    fn world(configured: bool) -> World {
        World {
            configured,
            accounts: HashMap::new(),
            ssh: false,
            feeds: Vec::new(),
            trust_level: 0,
            downloads: Vec::new(),
        }
    }

    #[test]
    fn sign_in_needs_the_code_when_asked_to() {
        let options = Options {
            port: 0,
            otp: true,
            unconfigured: false,
        };
        let mut world = world(true);
        let login = |extra: &[(&str, &str)], world: &mut World| {
            let mut pairs = vec![
                ("api", "SYNO.API.Auth"),
                ("method", "login"),
                ("account", "demo"),
                ("passwd", "demo"),
            ];
            pairs.extend_from_slice(extra);
            answer(&params(&pairs), false, &options, world)
        };
        assert_eq!(login(&[], &mut world)["error"]["code"], 403);
        assert_eq!(
            login(&[("otp_code", "1")], &mut world)["error"]["code"],
            404
        );
        assert_eq!(
            login(&[("otp_code", OTP)], &mut world)["data"]["did"],
            DEVICE_TOKEN
        );
        assert_eq!(
            login(&[("device_id", DEVICE_TOKEN)], &mut world)["success"],
            true
        );
    }

    #[test]
    fn setup_creates_an_account_and_ends_the_setup_mode() {
        let options = Options {
            port: 0,
            otp: false,
            unconfigured: true,
        };
        let mut world = world(false);
        let check = params(&[
            ("api", "SYNO.Core.QuickStart.Info"),
            ("method", "check_permission"),
        ]);
        assert_eq!(answer(&check, true, &options, &mut world)["success"], true);

        let compound = r#"[{"api":"SYNO.Core.User","method":"create","version":1,"name":"owner","password":"Secret-123"},
                           {"api":"SYNO.Core.User","method":"set","version":1,"name":"admin","expired":"now"}]"#;
        let setup = params(&[
            ("api", "SYNO.Entry.Request"),
            ("method", "request"),
            ("compound", compound),
        ]);
        assert_eq!(
            answer(&setup, true, &options, &mut world)["data"]["has_fail"],
            false
        );
        assert!(world.configured);
        assert_eq!(
            answer(&check, true, &options, &mut world)["error"]["code"],
            119
        );

        let login = params(&[
            ("api", "SYNO.API.Auth"),
            ("method", "login"),
            ("account", "owner"),
            ("passwd", "Secret-123"),
        ]);
        assert_eq!(
            answer(&login, false, &options, &mut world)["data"]["sid"],
            SID
        );
    }

    #[test]
    fn data_needs_a_session() {
        let options = Options {
            port: 0,
            otp: false,
            unconfigured: false,
        };
        let mut world = world(true);
        let info = params(&[("api", "SYNO.Core.System"), ("method", "info")]);
        assert_eq!(
            answer(&info, false, &options, &mut world)["error"]["code"],
            119
        );
        let mut signed = info.clone();
        signed.insert("_sid".into(), SID.into());
        assert_eq!(
            answer(&signed, false, &options, &mut world)["data"]["model"],
            "DS923+"
        );
    }
}
