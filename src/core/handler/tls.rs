use std::sync::{Arc, OnceLock};

use chrono::{DateTime, Duration, Utc};
use rustls::{ClientConfig, RootCertStore};
use tokio::net::TcpStream;
use tokio_rustls::TlsStream;

const TLS_TIMEOUT: Duration = Duration::seconds(2);

static TLS_CONFIG: OnceLock<Arc<ClientConfig>> = OnceLock::new();

pub struct TlsHandshakeOutcome {
    pub stream: TlsStream<TcpStream>,
    pub cert_expires_at: Option<DateTime<Utc>>,
}

#[allow(clippy::expect_used)]
fn build_tls_config() -> Arc<ClientConfig> {
    TLS_CONFIG
        .get_or_init(|| {
            rustls::crypto::ring::default_provider()
                .install_default()
                .expect("failed to install rustls crypto provider");

            let mut root_store = RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

            let config = ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();

            Arc::new(config)
        })
        .clone()
}

pub async fn handshake(
    tcp_stream: TcpStream,
    domain: &str,
) -> Result<TlsHandshakeOutcome, anyhow::Error> {
    let config = build_tls_config();
    let server_name = rustls::pki_types::ServerName::try_from(domain.to_string())?;

    let connector = tokio_rustls::TlsConnector::from(config);
    let tls_stream = tokio::time::timeout(
        TLS_TIMEOUT.to_std()?,
        connector.connect(server_name, tcp_stream),
    )
    .await
    .map_err(|e| anyhow::anyhow!(e))??;

    let cert_expires_at = tls_stream
        .get_ref()
        .1
        .peer_certificates()
        .into_iter()
        .flatten()
        .find_map(|cert| {
            x509_parser::parse_x509_certificate(cert.as_ref())
                .ok()
                .and_then(|(_, parsed)| {
                    DateTime::from_timestamp(
                        parsed.tbs_certificate.validity.not_after.timestamp(),
                        0,
                    )
                })
        });

    Ok(TlsHandshakeOutcome {
        stream: tls_stream.into(),
        cert_expires_at,
    })
}
