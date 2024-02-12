use crate::{config::ConfigInner, tcp::process::process_tcp};
use anyhow::Result;
use rustls::ServerConfig;
use std::sync::Arc;
use tokio::net::TcpListener;
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
            if let Ok(incoming) = acceptor.accept(incoming).await {
                let config_clone = self.config.clone();

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
