use crate::{config::Config, tcp::process::process_tcp};
use anyhow::{anyhow, Result};
use rustls::ServerConfig;
use std::{fs::File, io::BufReader, sync::Arc};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

pub struct PtmListener {
    listen: String,
    config: Arc<Config>,
    ssl_server_config: Option<Arc<ServerConfig>>,
}

impl PtmListener {
    pub fn from_config(config: Config) -> Result<Self> {
        let listen = config.server.listen.clone();
        let ssl_enabled = config.server.ssl.is_some_and(|ssl| ssl == true).clone();
        let ssl_server_config: Option<Arc<ServerConfig>>;

        if ssl_enabled {
            let cert_path = config
                .server
                .cert_path
                .as_ref()
                .ok_or(anyhow!("Missing cert_path"))?;
            let key_path = config
                .server
                .key_path
                .as_ref()
                .ok_or(anyhow!("Missing key_path"))?;

            let certs = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(cert_path)?))
                .collect::<Result<Vec<_>, _>>()?;
            let private_key =
                rustls_pemfile::private_key(&mut BufReader::new(&mut File::open(key_path)?))?
                    .ok_or(anyhow!("Invalid key"))?;
            ssl_server_config = Some(Arc::new(
                rustls::ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(certs, private_key)?,
            ));
        } else {
            ssl_server_config = None
        }

        let config = Arc::new(config);
        Ok(PtmListener {
            listen,
            config,
            ssl_server_config,
        })
    }

    pub async fn start(&self) -> Result<()> {
        if let Some(ssl_config) = self.ssl_server_config.clone() {
            self.tls_listener(ssl_config).await
        } else {
            self.tcp_listener().await
        }
    }

    async fn tls_listener(&self, ssl_config: Arc<ServerConfig>) -> Result<()> {
        let acceptor = TlsAcceptor::from(ssl_config);
        let listener = TcpListener::bind(&self.listen).await?;
        tracing::info!("Listening on {} (tls)", listener.local_addr()?);

        while let Ok((incoming, _)) = listener.accept().await {
            if let Ok(mut incoming) = acceptor.accept(incoming).await {
                let config_clone = self.config.clone();

                tokio::spawn(async move {
                    if let Err(e) = process_tcp(&mut incoming, config_clone).await {
                        tracing::error!("{e}");
                    }
                });
            }
        }

        Ok(())
    }

    async fn tcp_listener(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.listen).await?;
        tracing::info!("Listening on {}", listener.local_addr()?);

        while let Ok((mut incoming, _)) = listener.accept().await {
            let config_clone = self.config.clone();

            tokio::spawn(async move {
                if let Err(e) = process_tcp(&mut incoming, config_clone).await {
                    tracing::error!("{e}");
                }
            });
        }

        Ok(())
    }
}
