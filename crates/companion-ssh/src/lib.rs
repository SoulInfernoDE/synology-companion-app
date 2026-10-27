//! SSH to a Synology NAS.
//!
//! The host key is pinned like the HTTPS certificate: the first connection reports
//! its fingerprint, the user trusts it, and later connections must present exactly
//! that key. Keys are ed25519 in OpenSSH format, so they also work with `ssh`.
//! `sudo` gets the password on its standard input, never on a command line, so it
//! does not show up in the process list or the shell history of the NAS.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use russh::client::{self, Handle};
use russh::keys::{HashAlg, PrivateKey, PrivateKeyWithHashAlg, PublicKey, PublicKeyOrCertificate};
use russh::{ChannelMsg, Disconnect};
use tokio::sync::mpsc;

pub mod containers;
pub mod diagnostics;
pub mod packages;

/// Adds the public key on standard input to `authorized_keys`, once, with the
/// permissions sshd insists on. Exits with 42 when the account has no home folder.
const INSTALL_KEY: &str = r#"[ -d "$HOME" ] || exit 42
umask 077
read -r key || exit 1
mkdir -p "$HOME/.ssh" && touch "$HOME/.ssh/authorized_keys" || exit 1
grep -qxF "$key" "$HOME/.ssh/authorized_keys" || printf '%s\n' "$key" >> "$HOME/.ssh/authorized_keys"
chmod 700 "$HOME/.ssh" && chmod 600 "$HOME/.ssh/authorized_keys" && chmod go-w "$HOME""#;

/// Removes the public key on standard input from `authorized_keys`.
const REMOVE_KEY: &str = r#"read -r key || exit 1
file="$HOME/.ssh/authorized_keys"
[ -f "$file" ] || exit 0
grep -vxF "$key" "$file" > "$file.syno-companion"
cat "$file.syno-companion" > "$file" && rm -f "$file.syno-companion""#;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("cannot reach the NAS over SSH: {0}")]
    Network(String),
    #[error("the SSH host key is not trusted yet ({fingerprint})")]
    UnknownHostKey { fingerprint: String },
    #[error("the SSH host key changed ({fingerprint})")]
    HostKeyChanged { fingerprint: String },
    #[error("the NAS refused the sign-in")]
    AuthFailed,
    #[error("SSH: {0}")]
    Protocol(String),
    #[error("invalid key: {0}")]
    Key(String),
    #[error("the account has no home folder on the NAS")]
    NoHome,
}

impl Error {
    /// A stable, kebab-case key for the user interface to pick its message.
    pub fn key(&self) -> &'static str {
        match self {
            Self::Network(_) => "ssh-network",
            Self::UnknownHostKey { .. } => "ssh-host-unknown",
            Self::HostKeyChanged { .. } => "ssh-host-changed",
            Self::AuthFailed => "ssh-auth",
            Self::NoHome => "ssh-no-home",
            Self::Protocol(_) | Self::Key(_) => "ssh-failed",
        }
    }
}

impl From<russh::Error> for Error {
    fn from(error: russh::Error) -> Self {
        Self::Protocol(error.to_string())
    }
}

/// "SHA256:…", as `ssh-keygen -l` shows it.
pub fn fingerprint(key: &PublicKey) -> String {
    key.fingerprint(HashAlg::Sha256).to_string()
}

/// A new ed25519 key: the private key in OpenSSH format and the public key line.
pub fn generate_key(comment: &str) -> Result<(String, String), Error> {
    let mut seed = [0u8; 32];
    ring::rand::SecureRandom::fill(&ring::rand::SystemRandom::new(), &mut seed)
        .map_err(|_| Error::Key("no system randomness".into()))?;
    let pair = russh::keys::ssh_key::private::Ed25519Keypair::from_seed(&seed);
    let mut key = PrivateKey::from(pair);
    key.set_comment(comment);
    let private = key
        .to_openssh(russh::keys::ssh_key::LineEnding::LF)
        .map_err(|error| Error::Key(error.to_string()))?
        .to_string();
    let public = key
        .public_key()
        .to_openssh()
        .map_err(|error| Error::Key(error.to_string()))?;
    Ok((private, public))
}

/// The public key line of a private key in OpenSSH format.
pub fn public_key(private: &str) -> Result<String, Error> {
    PrivateKey::from_openssh(private)
        .map_err(|error| Error::Key(error.to_string()))?
        .public_key()
        .to_openssh()
        .map_err(|error| Error::Key(error.to_string()))
}

/// Quotes `value` for a POSIX shell.
pub fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

/// How to prove who we are.
pub enum Auth {
    /// A private key in OpenSSH format.
    Key(String),
    /// The DSM password; used once, to install a key.
    Password(String),
}

struct Client {
    pinned: Option<String>,
    seen: Arc<Mutex<Option<String>>>,
}

impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        let PublicKeyOrCertificate::PublicKey { key, .. } = server_public_key else {
            return Ok(false);
        };
        let presented = fingerprint(key);
        let trusted = self.pinned.as_deref() == Some(presented.as_str());
        if let Ok(mut seen) = self.seen.lock() {
            *seen = Some(presented);
        }
        Ok(trusted)
    }
}

/// What a command printed and how it ended.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Output {
    pub status: Option<u32>,
    pub stdout: String,
    pub stderr: String,
}

impl Output {
    pub fn success(&self) -> bool {
        self.status == Some(0)
    }
}

/// A signed-in SSH connection.
pub struct Connection {
    handle: Handle<Client>,
}

impl Connection {
    /// Connects and signs in. `pinned` is the host key fingerprint the user trusted.
    pub async fn open(
        host: &str,
        port: u16,
        user: &str,
        auth: Auth,
        pinned: Option<&str>,
    ) -> Result<Self, Error> {
        let config = Arc::new(client::Config {
            inactivity_timeout: Some(Duration::from_secs(600)),
            keepalive_interval: Some(Duration::from_secs(30)),
            ..client::Config::default()
        });
        let seen = Arc::new(Mutex::new(None));
        let handler = Client {
            pinned: pinned.map(str::to_owned),
            seen: seen.clone(),
        };
        let connect = client::connect(config, (host, port), handler);
        let result = tokio::time::timeout(Duration::from_secs(10), connect)
            .await
            .map_err(|_| Error::Network("timed out".into()))?;
        let mut handle = match result {
            Ok(handle) => handle,
            Err(error) => {
                let presented = seen.lock().ok().and_then(|mut seen| seen.take());
                return Err(match (presented, pinned) {
                    (Some(fingerprint), None) => Error::UnknownHostKey { fingerprint },
                    (Some(fingerprint), Some(pin)) if fingerprint != pin => {
                        Error::HostKeyChanged { fingerprint }
                    }
                    _ => Error::Network(error.to_string()),
                });
            }
        };

        let result = match auth {
            Auth::Key(private) => {
                let key = PrivateKey::from_openssh(&private)
                    .map_err(|error| Error::Key(error.to_string()))?;
                let hash = handle.best_supported_rsa_hash().await?.flatten();
                handle
                    .authenticate_publickey(user, PrivateKeyWithHashAlg::new(Arc::new(key), hash))
                    .await?
            }
            Auth::Password(password) => {
                let result = handle.authenticate_password(user, password.clone()).await?;
                if result.success() {
                    result
                } else {
                    // DSM may offer keyboard-interactive instead of plain password.
                    keyboard_interactive(&mut handle, user, &password).await?
                }
            }
        };
        if !result.success() {
            return Err(Error::AuthFailed);
        }
        Ok(Self { handle })
    }

    async fn exec(&self, command: &str, input: Option<&[u8]>) -> Result<Output, Error> {
        let mut channel = self.handle.channel_open_session().await?;
        channel.exec(true, command).await?;
        if let Some(input) = input {
            channel.data(input).await?;
        }
        channel.eof().await?;
        let mut output = Output::default();
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        while let Some(message) = channel.wait().await {
            match message {
                ChannelMsg::Data { data } => stdout.extend_from_slice(&data),
                ChannelMsg::ExtendedData { data, ext: 1 } => stderr.extend_from_slice(&data),
                ChannelMsg::ExitStatus { exit_status } => output.status = Some(exit_status),
                _ => {}
            }
        }
        output.stdout = String::from_utf8_lossy(&stdout).into_owned();
        output.stderr = String::from_utf8_lossy(&stderr).into_owned();
        Ok(output)
    }

    /// Runs `command` in the user's shell.
    pub async fn run(&self, command: &str) -> Result<Output, Error> {
        self.exec(command, None).await
    }

    /// Runs `command` and writes `input` to its standard input.
    pub async fn run_with_input(&self, command: &str, input: &[u8]) -> Result<Output, Error> {
        self.exec(command, Some(input)).await
    }

    /// Runs `command` as root. `-k` makes `sudo` always read the password, so it
    /// never reaches the command. The password reaches `sudo` on its standard input,
    /// so it never appears on a command line; the command itself gets no input.
    pub async fn run_sudo(&self, command: &str, password: &str) -> Result<Output, Error> {
        let wrapped = format!(
            "sudo -k -S -p '' -- sh -c {}",
            quote(&format!("exec 0</dev/null; {command}"))
        );
        let mut input = password.as_bytes().to_vec();
        input.push(b'\n');
        self.exec(&wrapped, Some(&input)).await
    }

    /// Runs `command` as root and gives it `input` on standard input, after the
    /// password line that `sudo` reads.
    pub async fn run_sudo_with_input(
        &self,
        command: &str,
        password: &str,
        input: &[u8],
    ) -> Result<Output, Error> {
        let wrapped = format!("sudo -k -S -p '' -- sh -c {}", quote(command));
        let mut data = password.as_bytes().to_vec();
        data.push(b'\n');
        data.extend_from_slice(input);
        self.exec(&wrapped, Some(&data)).await
    }

    /// An interactive shell with a terminal of `cols` × `rows`.
    pub async fn shell(&self, cols: u32, rows: u32) -> Result<Shell, Error> {
        let channel = self.handle.channel_open_session().await?;
        channel
            .request_pty(false, "xterm-256color", cols, rows, 0, 0, &[])
            .await?;
        channel.request_shell(false).await?;
        let (output_tx, output_rx) = mpsc::unbounded_channel();
        let (input_tx, mut input_rx) = mpsc::unbounded_channel::<ShellInput>();
        tokio::spawn(async move {
            let mut channel = channel;
            loop {
                tokio::select! {
                    message = channel.wait() => match message {
                        Some(ChannelMsg::Data { data } | ChannelMsg::ExtendedData { data, .. }) => {
                            if output_tx.send(data.to_vec()).is_err() {
                                break;
                            }
                        }
                        Some(ChannelMsg::Eof | ChannelMsg::Close) | None => break,
                        Some(_) => {}
                    },
                    input = input_rx.recv() => match input {
                        Some(ShellInput::Data(bytes)) => {
                            if channel.data(bytes.as_slice()).await.is_err() {
                                break;
                            }
                        }
                        Some(ShellInput::Resize(cols, rows)) => {
                            let _ = channel.window_change(cols, rows, 0, 0).await;
                        }
                        Some(ShellInput::Close) | None => {
                            let _ = channel.close().await;
                            break;
                        }
                    },
                }
            }
        });
        Ok(Shell {
            input: input_tx,
            output: output_rx,
        })
    }

    /// Lets `public_key` (an OpenSSH line) sign in to this account.
    pub async fn install_key(&self, public_key: &str) -> Result<(), Error> {
        let output = self
            .run_with_input(INSTALL_KEY, format!("{public_key}\n").as_bytes())
            .await?;
        match output.status {
            Some(0) => Ok(()),
            Some(42) => Err(Error::NoHome),
            _ => Err(Error::Protocol(output.stderr.trim().to_owned())),
        }
    }

    /// Takes `public_key` out of this account's `authorized_keys`.
    pub async fn remove_key(&self, public_key: &str) -> Result<(), Error> {
        let output = self
            .run_with_input(REMOVE_KEY, format!("{public_key}\n").as_bytes())
            .await?;
        if output.success() {
            Ok(())
        } else {
            Err(Error::Protocol(output.stderr.trim().to_owned()))
        }
    }

    /// Whether the NAS or the network has ended the connection.
    pub fn is_closed(&self) -> bool {
        self.handle.is_closed()
    }

    pub async fn close(self) {
        let _ = self
            .handle
            .disconnect(Disconnect::ByApplication, "", "en")
            .await;
    }
}

async fn keyboard_interactive(
    handle: &mut Handle<Client>,
    user: &str,
    password: &str,
) -> Result<russh::client::AuthResult, Error> {
    use russh::client::KeyboardInteractiveAuthResponse as Response;
    let mut response = handle
        .authenticate_keyboard_interactive_start(user, None::<String>)
        .await?;
    for _ in 0..3 {
        match response {
            Response::Success => return Ok(russh::client::AuthResult::Success),
            Response::Failure { .. } => break,
            Response::InfoRequest { ref prompts, .. } => {
                let answers = prompts.iter().map(|_| password.to_owned()).collect();
                response = handle
                    .authenticate_keyboard_interactive_respond(answers)
                    .await?;
            }
        }
    }
    Err(Error::AuthFailed)
}

enum ShellInput {
    Data(Vec<u8>),
    Resize(u32, u32),
    Close,
}

/// An interactive shell: write keystrokes, read what the NAS prints.
pub struct Shell {
    input: mpsc::UnboundedSender<ShellInput>,
    output: mpsc::UnboundedReceiver<Vec<u8>>,
}

/// The writing half of a [`Shell`], to keep while another task reads.
#[derive(Clone)]
pub struct ShellWriter {
    input: mpsc::UnboundedSender<ShellInput>,
}

impl ShellWriter {
    pub fn write(&self, bytes: &[u8]) -> bool {
        self.input.send(ShellInput::Data(bytes.to_vec())).is_ok()
    }

    pub fn resize(&self, cols: u32, rows: u32) {
        let _ = self.input.send(ShellInput::Resize(cols, rows));
    }

    pub fn close(&self) {
        let _ = self.input.send(ShellInput::Close);
    }
}

impl Shell {
    /// Splits into a writer and the stream of output, which ends with the shell.
    pub fn split(self) -> (ShellWriter, mpsc::UnboundedReceiver<Vec<u8>>) {
        (ShellWriter { input: self.input }, self.output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_keys_round_trip() {
        let (private, public) = generate_key("syno-companion").unwrap();
        assert!(private.starts_with("-----BEGIN OPENSSH PRIVATE KEY-----"));
        assert!(public.starts_with("ssh-ed25519 "));
        assert!(public.ends_with(" syno-companion"));
        assert_eq!(public_key(&private).unwrap(), public);
        assert_ne!(generate_key("x").unwrap().1, generate_key("x").unwrap().1);
    }

    #[test]
    fn quotes_for_the_shell() {
        assert_eq!(quote("plain"), "'plain'");
        assert_eq!(quote("it's"), r"'it'\''s'");
    }
}
