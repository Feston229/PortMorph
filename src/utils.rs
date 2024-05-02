use crate::config::ConfigInner;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub fn init_tracing() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

// Get to which address to forward by location name
pub async fn get_forward_by_name(config: &Arc<ConfigInner>, name: &str) -> Result<String> {
    let addr: String = config
        .location
        .clone()
        .into_iter()
        .filter(|loc| loc.name == name)
        .map(|loc| loc.forward_to)
        .collect();
    Ok(addr)
}

// Get to which address to forward by location path
pub async fn get_forward_by_path(config: &Arc<ConfigInner>, path: &str) -> Result<String> {
    let addr: String = config
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
    config: &Arc<ConfigInner>,
    request_path: &str,
    method: &str,
) -> Result<String> {
    let paths: Vec<String> = config
        .location
        .clone()
        .into_iter()
        .filter(|loc| loc.path != "/")
        .map(|loc| loc.path)
        .collect();
    tracing::info!("paths -> {:?}", paths);
    if let Some(matching_path) = paths
        .into_iter()
        .find(|path| request_path.starts_with(&path.to_owned()))
    {
        Ok(matching_path)
    } else {
        Err(anyhow!("Missing route"))
    }
}
