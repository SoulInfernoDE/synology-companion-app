//! Runs the client once against a DSM, for example the mock:
//!
//! ```text
//! cargo run -p companion-mockdsm &
//! cargo run -p companion-dsm --example smoke -- 127.0.0.1 15001 <fingerprint> demo demo
//! ```

use companion_dsm::{Client, Credentials, SetupRequest, SetupState, UpdatePolicy};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [host, port, fingerprint, account, password] = args.as_slice() else {
        eprintln!("usage: smoke HOST PORT FINGERPRINT ACCOUNT PASSWORD");
        std::process::exit(2);
    };
    let port: u16 = port.parse().expect("port");

    let unpinned = Client::new(host, port, None).expect("client");
    println!(
        "without pin: {:?}",
        unpinned.setup_state().await.map_err(|e| e.key())
    );

    let client = Client::new(host, port, Some(fingerprint)).expect("client");
    let state = client.setup_state().await;
    println!("setup state: {state:?}");
    if state == Ok(SetupState::NotConfigured) {
        let info = client.begin_setup().await.expect("setup info");
        println!(
            "setup info: {} zones, rules {:?}",
            info.time_zones.len(),
            info.password_rules
        );
        let done = client
            .complete_setup(&SetupRequest {
                device_name: "smoke".into(),
                account: account.clone(),
                password: password.clone(),
                time_zone: "Amsterdam".into(),
                updates: UpdatePolicy::Important,
                device_label: "smoke test".into(),
            })
            .await
            .expect("setup");
        println!(
            "setup done, signed in: {}, skipped {:?}",
            done.session.is_some(),
            done.skipped
        );
        println!(
            "setup state after: {:?}",
            Client::new(host, port, Some(fingerprint))
                .expect("client")
                .setup_state()
                .await
        );
    }
    let session = client
        .login(&Credentials {
            account: account.clone(),
            password: password.clone(),
            device_name: "smoke test".into(),
            otp: std::env::var("OTP").ok(),
            ..Credentials::default()
        })
        .await
        .expect("sign in");
    let overview = client.overview(&session).await.expect("overview");
    println!(
        "overview: {} DSM {} cpu {:?} ({:?}) {} volumes {} drives {} ports",
        overview.model,
        overview.dsm,
        overview.cpu_model,
        overview.cpu_architecture,
        overview.volumes.len(),
        overview.drives.len(),
        overview.network_ports.len()
    );
    println!("identity: {:?}", client.identity(&session).await);
    println!("updates: {:?}", client.updates(&session).await);
    println!("shares: {:?}", client.shares(&session).await);
    println!("beep: {:?}", client.stop_beep(&session).await);
    println!("reboot: {:?}", client.reboot(&session).await);
    client.logout(&session).await;
}
