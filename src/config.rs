use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::Deserialize;
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

pub async fn init_config() -> Result<Config> {
    let mut toml_content = String::new();
    if let Ok(mut file) = File::open("ptm.toml").await {
        file.read_to_string(&mut toml_content).await.ok();
    }

    let config: Config = from_str(&toml_content)?;

    // Validate config
    if let Some(ssl) = config.server.ssl {
        if ssl {
            if let Some(cert_path) = &config.server.cert_path {
                if !PathBuf::from(cert_path).exists() {
                    return Err(anyhow!(format!("{cert_path} file not found")));
                }
            } else {
                return Err(anyhow!("cert_path is missing in configuration"));
            }
            if let Some(key_path) = &config.server.key_path {
                if !PathBuf::from(key_path).exists() {
                    return Err(anyhow!(format!("{key_path} file not found")));
                }
            } else {
                return Err(anyhow!("key_path is missing in configuration"));
            }
        }
    }

    Ok(config)
}
