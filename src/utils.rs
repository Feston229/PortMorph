use std::sync::Arc;

use anyhow::{anyhow, Result};
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

// Get to which address to forward by location name
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

// Get to which address to forward by location path
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

// Find path related to request
pub async fn find_path(
    config: &Arc<Mutex<Config>>,
    request_path: &str,
    method: &str,
) -> Result<String> {
    let paths: Vec<String> = config
        .lock()
        .await
        .location
        .clone()
        .into_iter()
        .filter(|loc| {
            loc.path != "/" && loc.method == Some(method.to_owned()) || loc.method == None
        })
        .map(|loc| loc.path)
        .collect();
    if let Some(matching_path) = paths
        .into_iter()
        .find(|path| request_path.starts_with(&*path))
    {
        return Ok(matching_path);
    }
    return Err(anyhow!("Missing route"));
}
