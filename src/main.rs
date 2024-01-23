mod config;
mod controllers;
mod utils;

use controllers::run;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run().await.unwrap_or_else(|e| tracing::error!("{e}"));
}
