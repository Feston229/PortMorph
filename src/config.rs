use anyhow::Result;
use rustls::ServerConfig;
use serde::Deserialize;
use std::{io::BufReader, sync::Arc};
use tokio::{fs::File, io::AsyncReadExt};
use toml::from_str;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    pub location: Vec<Location>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub listen: String,
    pub ssl: Option<bool>,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Location {
    pub name: String,
    pub path: String,
    pub forward_to: String,
    pub method: Option<String>,
}

pub struct ConfigInner {
    pub server: ServerInner,
    pub location: Vec<Location>,
}

#[derive(Debug)]
pub struct ServerInner {
    pub listen: String,
    pub server_config: Option<Arc<ServerConfig>>,
}

impl From<Config> for ConfigInner {
    fn from(config: Config) -> Self {
        let server: ServerInner;
        let location = config.location;

        let listen = config.server.listen;
        let server_config: Option<Arc<ServerConfig>>;

        if config.server.ssl.is_some_and(|ssl| ssl == true)
            && config.server.cert_path.is_some()
            && config.server.key_path.is_some()
        {
            let cert_path = config.server.cert_path.as_ref().expect("Missing cert_path");
            let key_path = config.server.key_path.as_ref().expect("Missing key_path");

            let certs = rustls_pemfile::certs(&mut BufReader::new(
                &mut std::fs::File::open(cert_path).expect("Failed to open cert_path"),
            ))
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to load cert");

            let private_key = rustls_pemfile::private_key(&mut BufReader::new(
                &mut std::fs::File::open(key_path).expect("Failed to open key_path"),
            ))
            .expect("Failed to load private key in file")
            .expect("Failed to find private key in file");

            server_config = Some(Arc::new(
                rustls::ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(certs, private_key)
                    .expect("Failed to build rustls config"),
            ));
        } else {
            server_config = None;
        }

        server = ServerInner {
            listen,
            server_config,
        };

        ConfigInner { server, location }
    }
}

pub async fn load_config() -> Result<ConfigInner> {
    let mut toml_content = String::new();
    if let Ok(mut file) = File::open("ptm.toml").await {
        file.read_to_string(&mut toml_content).await.ok();
    }

    let config: Config = from_str(&toml_content)?;
    let config_inner = ConfigInner::from(config);

    Ok(config_inner)
}
