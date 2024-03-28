use crate::{config::load_config, tcp::listener::PtmListener, utils::init_tracing};
use anyhow::Result;
use std::path::PathBuf;

mod config;
mod tcp;
#[cfg(feature = "tests")]
mod test;
mod utils;

#[tokio::main]
async fn main() {
    run().await.unwrap_or_else(|e| tracing::error!("{e}"));
}

pub async fn run() -> Result<()> {
    init_tracing()?;
    let config_path: String;
    if PathBuf::from("ptm.toml").exists() {
        config_path = String::from("ptm.toml");
    } else {
        config_path = String::from("/etc/ptm/ptm.toml")
    }
    let config = load_config(&config_path).await?;

    let listener = PtmListener::new(config);
    listener.start().await
}
