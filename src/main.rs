mod config;
mod tcp;
mod utils;

use crate::{config::init_config, tcp::listener::PtmListener, utils::init_tracing};
use anyhow::Result;

#[tokio::main]
async fn main() {
    run().await.unwrap_or_else(|e| tracing::error!("{e}"));
}

pub async fn run() -> Result<()> {
    init_tracing()?;
    let config = init_config().await?;

    let listener = PtmListener::from_config(config)?;
    listener.start().await
}
