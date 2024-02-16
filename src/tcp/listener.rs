use crate::{config::ConfigInner, tcp::process::process_tcp};
use anyhow::Result;
use rustls::ServerConfig;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;

pub struct PtmListener {
    config: Arc<ConfigInner>,
}

impl PtmListener {
    pub fn new(config: ConfigInner) -> Self {
        let config = Arc::new(config);
        Self { config }
    }

    pub async fn start(&self) -> Result<()> {
        if let Some(server_config) = self.config.server.server_config.clone() {
            self.tls_listener(server_config).await
        } else {
            self.tcp_listener().await
        }
    }

    async fn tls_listener(&self, server_config: Arc<ServerConfig>) -> Result<()> {
        let acceptor = TlsAcceptor::from(server_config);
        let listener = TcpListener::bind(&self.config.server.listen).await?;
        tracing::info!("Listening on {} (tls)", listener.local_addr()?);

        while let Ok((incoming, _)) = listener.accept().await {
            let config_clone = self.config.clone();

            if tls_detected(&incoming).await? {
                match acceptor.accept(incoming).await {
                    Ok(incoming) => {
                        tokio::spawn(async move {
                            if let Err(e) = process_tcp(incoming, config_clone).await {
                                tracing::error!("{e}");
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to accept tls stream: {e}");
                    }
                }
            } else {
                tokio::spawn(async move {
                    if let Err(e) = process_tcp(incoming, config_clone).await {
                        tracing::error!("{e}");
                    }
                });
            }
        }

        Ok(())
    }

    async fn tcp_listener(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.config.server.listen).await?;
        tracing::info!("Listening on {}", listener.local_addr()?);

        while let Ok((incoming, _)) = listener.accept().await {
            let config_clone = self.config.clone();

            tokio::spawn(async move {
                if let Err(e) = process_tcp(incoming, config_clone).await {
                    tracing::error!("{e}");
                }
            });
        }

        Ok(())
    }
}

async fn tls_detected(stream: &TcpStream) -> Result<bool> {
    let mut peek_bytes = [0u8; 8];
    stream.peek(&mut peek_bytes).await?;

    if peek_bytes.starts_with(b"\x16\x03") {
        return Ok(true);
    }
    Ok(false)
}
