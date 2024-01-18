use anyhow::Result;
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
}

#[derive(Debug, Deserialize, Clone)]
pub struct Location {
    pub name: String,
    pub path: String,
    pub forward_to: String,
}

pub async fn init_config() -> Result<Config> {
    let mut toml_content = String::new();
    if let Ok(mut file) = File::open("ptm.toml").await {
        file.read_to_string(&mut toml_content).await.ok();
    }

    let config: Config = from_str(&toml_content)?;

    Ok(config)
}
