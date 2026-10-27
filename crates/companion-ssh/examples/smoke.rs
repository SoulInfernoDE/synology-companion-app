//! Signs in with a key and runs a few commands.
//!
//! `cargo run -p companion-ssh --example smoke -- <host> <port> <user> <key file> [fingerprint]`
//! Without a fingerprint it prints the host key's and stops. With `--new-key <file>` it
//! writes a fresh key pair and prints the public key.

use companion_ssh::{Auth, Connection, Error, generate_key};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--new-key") {
        let (private, public) = generate_key("syno-companion-smoke").unwrap();
        std::fs::write(&args[1], private).unwrap();
        println!("{public}");
        return;
    }
    let [host, port, user, key, rest @ ..] = args.as_slice() else {
        eprintln!("usage: smoke <host> <port> <user> <key file> [fingerprint]");
        std::process::exit(2);
    };
    let port: u16 = port.parse().unwrap();
    let key = std::fs::read_to_string(key).unwrap();
    let pinned = rest.first().map(String::as_str);
    let connection = match Connection::open(host, port, user, Auth::Key(key), pinned).await {
        Ok(connection) => connection,
        Err(Error::UnknownHostKey { fingerprint }) => {
            println!("host key: {fingerprint}");
            return;
        }
        Err(error) => {
            eprintln!("{error} ({})", error.key());
            std::process::exit(1);
        }
    };
    for command in [
        "uname -r",
        "cat /etc.defaults/VERSION | grep -E 'productversion|buildnumber'",
        "echo err >&2; exit 3",
    ] {
        println!("$ {command}\n{:?}", connection.run(command).await.unwrap());
    }
    let sudo = connection
        .run_sudo("id -u", "wrong password")
        .await
        .unwrap();
    println!("sudo with a wrong password: {sudo:?}");
    let input = connection.run_with_input("wc -c", b"hello").await.unwrap();
    println!("input: {input:?}");
    println!("diagnostics: {:?}", connection.diagnostics().await.unwrap());
    let (_, extra) = companion_ssh::generate_key("syno-companion-extra").unwrap();
    connection.install_key(&extra).await.unwrap();
    connection.install_key(&extra).await.unwrap();
    let count = connection
        .run("grep -c syno-companion-extra ~/.ssh/authorized_keys")
        .await
        .unwrap();
    connection.remove_key(&extra).await.unwrap();
    let after = connection
        .run("grep -c syno-companion ~/.ssh/authorized_keys; stat -c %a ~/.ssh/authorized_keys")
        .await
        .unwrap();
    println!(
        "installed {} time(s); afterwards: {:?}",
        count.stdout.trim(),
        after.stdout
    );
    let (writer, mut output) = connection.shell(80, 24).await.unwrap().split();
    writer.write(b"stty size; echo shell-ok; exit\n");
    let mut seen = Vec::new();
    while let Some(bytes) = output.recv().await {
        seen.extend_from_slice(&bytes);
    }
    println!(
        "shell: {:?}",
        String::from_utf8_lossy(&seen).contains("shell-ok")
    );
    connection.close().await;
}
