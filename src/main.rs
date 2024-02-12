mod config;
mod tcp;
mod utils;

use crate::{config::load_config, tcp::listener::PtmListener, utils::init_tracing};
use anyhow::Result;

#[tokio::main]
async fn main() {
    run().await.unwrap_or_else(|e| tracing::error!("{e}"));
}

pub async fn run() -> Result<()> {
    init_tracing()?;
    let config = load_config().await?;

    let listener = PtmListener::new(config);
    listener.start().await
}
