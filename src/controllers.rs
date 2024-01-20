use std::sync::Arc;

use anyhow::{anyhow, Result};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{tcp, TcpListener, TcpStream},
    sync::Mutex,
    try_join,
};

use crate::{
    config::{init_config, Config},
    utils::{find_path, get_forward_by_name, get_forward_by_path, init_tracing},
};

pub async fn run() -> Result<()> {
    init_tracing()?;
    let config = Arc::new(Mutex::new(init_config().await?));
    let ssl_enabled = config
        .lock()
        .await
        .server
        .ssl
        .is_some_and(|ssl| ssl == true)
        .clone();

    if ssl_enabled {
        tls_listener(config).await?;
    } else {
        tcp_listener(config).await?;
    }

    Ok(())
}

// TODO
async fn tls_listener(config: Arc<Mutex<Config>>) -> Result<()> {
    Ok(())
}

async fn tcp_listener(config: Arc<Mutex<Config>>) -> Result<()> {
    let listener = TcpListener::bind(&config.lock().await.server.listen).await?;
    tracing::info!("Listening on {}", listener.local_addr()?);

    while let Ok((incoming, _)) = listener.accept().await {
        let config_clone = Arc::clone(&config);
        tokio::spawn(async move {
            if let Err(e) = process_conn(incoming, config_clone).await {
                tracing::error!("{e}");
            }
        });
    }

    Ok(())
}

async fn process_conn(mut incoming: TcpStream, config: Arc<Mutex<Config>>) -> Result<()> {
    let mut buf: Vec<u8> = vec![];
    let addr: String;
    incoming.read_buf(&mut buf).await?;

    // Process request
    let mut request = String::from_utf8_lossy(&buf);
    tracing::debug!("Got request -> {}", request);
    if request.starts_with("SSH") {
        tracing::debug!("Redirect to ssh");
        addr = get_forward_by_name(&config, "ssh").await?;
    } else if request.starts_with("GET") || request.starts_with("POST") {
        let method = request
            .split_whitespace()
            .nth(0)
            .ok_or(anyhow!("Unknown method"))?;
        let request_path = request
            .split_whitespace()
            .nth(1)
            .ok_or(anyhow!("Missing path"))?;
        let path = find_path(&config, request_path, method).await?;
        addr = get_forward_by_path(&config, &path).await?;
        request = request.replace(&format!(" {path}"), " ").into();
        buf = request.as_bytes().to_vec();
    } else {
        return Err(anyhow!("Unknown request"));
    }

    // Redirect
    let mut oncoming = TcpStream::connect(addr).await?;
    oncoming.write_all(&buf).await?;
    let (mut incoming_read, mut incoming_write) = incoming.split();
    let (mut oncoming_read, mut oncoming_write) = oncoming.split();
    let incoming_fut = io::copy(&mut incoming_read, &mut oncoming_write);
    let oncoming_fut = io::copy(&mut oncoming_read, &mut incoming_write);
    try_join!(incoming_fut, oncoming_fut)?;

    Ok(())
}
