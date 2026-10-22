use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::Arc;

use serde_json::json;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;

fn api_list() -> Value {
    json!({ "success": true, "data": {
        "SYNO.API.Auth": { "path": "entry.cgi", "minVersion": 1, "maxVersion": 7 },
        "SYNO.Core.System": { "path": "entry.cgi", "minVersion": 1, "maxVersion": 3 },
        "SYNO.Core.System.Utilization": { "path": "entry.cgi", "minVersion": 1, "maxVersion": 1 },
        "SYNO.Storage.CGI.Storage": { "path": "entry.cgi", "minVersion": 1, "maxVersion": 1 }
    }})
}

async fn mock_nas() -> (MockServer, Client) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/webapi/query.cgi"))
        .respond_with(ResponseTemplate::new(200).set_body_json(api_list()))
        .mount(&server)
        .await;
    let client = Client::with_base(Url::parse(&format!("{}/", server.uri())).unwrap());
    (server, client)
}

fn credentials() -> Credentials {
    Credentials {
        account: "admin".into(),
        password: "secret".into(),
        device_name: "Syno Companion test".into(),
        ..Credentials::default()
    }
}

#[tokio::test]
async fn otp_required_is_reported() {
    let (server, client) = mock_nas().await;
    Mock::given(path("/webapi/entry.cgi"))
        .and(body_string_contains("method=login"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "success": false, "error": { "code": 403 } })),
        )
        .mount(&server)
        .await;

    let error = client.login(&credentials()).await.unwrap_err();
    assert_eq!(error, Error::Auth(AuthError::OtpRequired));
    assert_eq!(error.key(), "otp-required");
}

#[tokio::test]
async fn otp_with_remember_device_returns_token() {
    let (server, client) = mock_nas().await;
    Mock::given(path("/webapi/entry.cgi"))
        .and(body_string_contains("method=login"))
        .and(body_string_contains("version=6"))
        .and(body_string_contains("otp_code=123456"))
        .and(body_string_contains("enable_device_token=yes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({ "success": true, "data": { "sid": "session-1", "did": "device-1" } }),
        ))
        .mount(&server)
        .await;

    let session = client
        .login(&Credentials {
            otp: Some("123456".into()),
            remember_device: true,
            ..credentials()
        })
        .await
        .unwrap();
    assert_eq!(session.sid, "session-1");
    assert_eq!(session.device_token.as_deref(), Some("device-1"));
}

#[tokio::test]
async fn wrong_password_and_expired_session_are_told_apart() {
    let (server, client) = mock_nas().await;
    Mock::given(path("/webapi/entry.cgi"))
        .and(body_string_contains("password-wrong"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "success": false, "error": { "code": 400 } })),
        )
        .mount(&server)
        .await;
    Mock::given(path("/webapi/entry.cgi"))
        .and(body_string_contains("method=login"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "success": true, "data": { "sid": "s" } })),
        )
        .mount(&server)
        .await;
    Mock::given(path("/webapi/entry.cgi"))
        .and(body_string_contains("api=SYNO.Core.System&"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "success": false, "error": { "code": 119 } })),
        )
        .mount(&server)
        .await;

    let wrong = Credentials {
        password: "password-wrong".into(),
        ..credentials()
    };
    assert_eq!(
        client.login(&wrong).await.unwrap_err(),
        Error::Auth(AuthError::WrongCredentials)
    );

    let session = client.login(&credentials()).await.unwrap();
    assert_eq!(
        client.overview(&session).await.unwrap_err(),
        Error::SessionExpired
    );
}

/// A one-request-per-connection HTTPS server with a self-signed certificate that
/// answers like DSM: the API list on `query.cgi`, a session on everything else.
fn self_signed_nas() -> (u16, String) {
    let certified = rcgen::generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
    let certificate = certified.cert.der().clone();
    let print = tls::fingerprint(&certificate);
    let key = rustls::pki_types::PrivateKeyDer::Pkcs8(certified.signing_key.serialize_der().into());
    let config = Arc::new(
        rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![certificate], key)
        .unwrap(),
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let connection = rustls::ServerConnection::new(config.clone()).unwrap();
            let mut tls = rustls::StreamOwned::new(connection, stream);
            let mut reader = BufReader::new(&mut tls);
            let mut request_line = String::new();
            if reader.read_line(&mut request_line).is_err() {
                continue; // the client refused the certificate
            }
            let mut length = 0;
            loop {
                let mut header = String::new();
                reader.read_line(&mut header).unwrap();
                if header.trim().is_empty() {
                    break;
                }
                if let Some(value) = header.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = value.trim().parse().unwrap();
                }
            }
            let mut body = vec![0; length];
            reader.read_exact(&mut body).unwrap();
            let answer = if request_line.contains("query.cgi") {
                api_list()
            } else {
                json!({ "success": true, "data": { "sid": "tls-session" } })
            }
            .to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{answer}",
                answer.len()
            );
            tls.write_all(response.as_bytes()).unwrap();
            tls.conn.send_close_notify();
            let _ = tls.flush();
        }
    });
    (port, print)
}

#[tokio::test]
async fn self_signed_certificate_needs_trust_then_pin_works() {
    let (port, print) = self_signed_nas();

    let unknown = Client::new("127.0.0.1", port, None).unwrap();
    assert_eq!(
        unknown.login(&credentials()).await.unwrap_err(),
        Error::UntrustedCertificate {
            fingerprint: print.clone()
        }
    );

    let pinned = Client::new("127.0.0.1", port, Some(&print)).unwrap();
    let session = pinned.login(&credentials()).await.unwrap();
    assert_eq!(session.sid, "tls-session");

    let other = "00:".repeat(31) + "00";
    let changed = Client::new("127.0.0.1", port, Some(&other)).unwrap();
    assert_eq!(
        changed.login(&credentials()).await.unwrap_err(),
        Error::CertificateChanged { fingerprint: print }
    );
}
