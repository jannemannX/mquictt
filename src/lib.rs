use std::{fs::File, io::BufReader, net::SocketAddr, sync::Arc};

use config::Auth;
use futures::StreamExt;
use quinn::{ClientConfig, TransportConfig};
use rustls::{Certificate, PrivateKey, RootCertStore};

mod client;
mod config;
mod error;
pub use client::Client;
pub use config::Config;
pub use error::Error;

pub(crate) struct Connection {
    conn: quinn::Connection,
    streams: quinn::IncomingBiStreams,
}

pub(crate) struct QuicServer {
    config: Arc<Config>,
    incoming: quinn::Incoming,
}

impl QuicServer {
    pub(crate) fn new(config: Arc<Config>, addr: &SocketAddr) -> Result<Self, Error> {
        let mut builder = quinn::Endpoint::builder().default_client_config(client_config(config)?);
        let (_, incoming) = builder.bind(addr)?;
        Ok(QuicServer { config, incoming })
    }

    pub(crate) async fn accept(&mut self) -> Result<Connection, Error> {
        let quinn::NewConnection {
            connection: conn,
            bi_streams: streams,
            ..
        } = match self.incoming.next().await {
            Some(connecting) => connecting.await?,
            None => return Err(Error::ConnectionBroken),
        };
        Ok(Connection { conn, streams })
    }
}

#[allow(dead_code)]
pub(crate) type QuicClient = Connection;

impl Connection {
    pub(crate) async fn connect(
        bind_addr: &SocketAddr,
        connect_addr: &SocketAddr,
        server_name: &str,
    ) -> Result<Self, Error> {
        let (endpoint, _) = quinn::Endpoint::builder().bind(bind_addr)?;
        let quinn::NewConnection {
            connection: conn,
            bi_streams: streams,
            ..
        } = endpoint.connect(connect_addr, server_name)?.await?;
        Ok(Connection { conn, streams })
    }

    pub(crate) async fn create_stream(
        &mut self,
    ) -> Result<(quinn::SendStream, quinn::RecvStream), Error> {
        Ok(self.conn.open_bi().await?)
    }

    pub(crate) async fn accept(&mut self) -> Result<(quinn::SendStream, quinn::RecvStream), Error> {
        match self.streams.next().await {
            Some(s) => Ok(s?),
            None => Err(Error::ConnectionBroken),
        }
    }
}

fn client_config(config: Arc<Config>) -> Result<ClientConfig, Error> {
    let client_config = match config.auth {
        Some(Auth {
            cert_file: cert_path,
            key_file: key_path,
            ca_cert_file: ca_path,
        }) => {
            // Get certificates
            let cert_file = File::open(&cert_path)?;
            let certs = rustls_pemfile::certs(&mut BufReader::new(cert_file))?;

            // Get private key
            let key_file = File::open(&key_path)?;
            let keys = rustls_pemfile::rsa_private_keys(&mut BufReader::new(key_file))?;

            // Get the first key
            let key = match keys.first() {
                Some(k) => k.clone(),
                None => return Err(Error::Tls),
            };

            let certs = certs
                .iter()
                .map(|cert| Certificate(cert.to_vec()))
                .collect();
            let key = PrivateKey(key);

            let ca_file = File::open(ca_path)?;
            let ca_file = rustls_pemfile::certs(&mut BufReader::new(ca_file))?;
            let mut store = RootCertStore::empty();
            store.add_parsable_certificates(&ca_file[..]);

            Ok(rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(store)
                .with_single_cert(certs, key)?)
        }

        None => Err(Error::Tls),
    };

    Ok(ClientConfig {
        transport: Arc::new(TransportConfig::default()),
        crypto: Arc::new(client_config?),
    })
}
