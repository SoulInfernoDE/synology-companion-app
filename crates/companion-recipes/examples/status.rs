//! Shows the status of every built-in recipe and the journal on a NAS.
//!
//! `cargo run -p companion-recipes --example status -- <host> <port> <user> <key file> <fingerprint>`

use companion_ssh::{Auth, Connection};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [host, port, user, key, fingerprint] = args.as_slice() else {
        eprintln!("usage: status <host> <port> <user> <key file> <fingerprint>");
        std::process::exit(2);
    };
    let key = std::fs::read_to_string(key).expect("key file");
    let connection = Connection::open(
        host,
        port.parse().expect("port"),
        user,
        Auth::Key(key),
        Some(fingerprint),
    )
    .await
    .expect("connect");
    for recipe in companion_recipes::builtin() {
        println!(
            "{} {}: {:?}",
            recipe.id,
            recipe.version,
            recipe.status(&connection).await
        );
    }
    println!(
        "journal: {:?}",
        companion_recipes::journal(&connection).await
    );
    connection.close().await;
}
