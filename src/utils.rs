use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::config::Config;

pub fn init_tracing() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

pub async fn get_forward_by_name(config: &Arc<Mutex<Config>>, name: &str) -> Result<String> {
    let addr: String = config
        .lock()
        .await
        .location
        .clone()
        .into_iter()
        .filter(|loc| loc.name == name)
        .map(|loc| loc.forward_to)
        .collect();
    Ok(addr)
}

pub async fn get_forward_by_path(config: &Arc<Mutex<Config>>, path: &str) -> Result<String> {
    let addr: String = config
        .lock()
        .await
        .location
        .clone()
        .into_iter()
        .filter(|loc| loc.path == path)
        .map(|loc| loc.forward_to)
        .collect();
    Ok(addr)
}
