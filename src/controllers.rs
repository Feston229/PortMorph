use std::{any, sync::Arc};

use anyhow::{anyhow, Context, Result};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    try_join,
};

use crate::{
    config::{init_config, Config},
    utils::{get_forward_by_name, get_forward_by_path, get_paths, init_tracing},
};

pub async fn run() -> Result<()> {
    init_tracing()?;
    let config = Arc::new(Mutex::new(init_config().await?));

    let listener = TcpListener::bind(&config.lock().await.server.listen).await?;
    tracing::info!("Listening on {}", listener.local_addr()?);

    while let Ok((incoming, _)) = listener.accept().await {
        let clone = Arc::clone(&config);
        tokio::spawn(async move {
            if let Err(e) = process_conn(incoming, clone).await {
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
    let request = String::from_utf8_lossy(&buf);
    tracing::debug!("Got request -> {}", request);

    if request.starts_with("SSH-") {
        tracing::debug!("Redirect to ssh");
        addr = get_forward_by_name(&config, "ssh").await?;
    } else if request.starts_with("GET") {
        let request_path = request.split_whitespace().nth(1).unwrap_or("/");
        tracing::debug!("Redirect to web with path {request_path}");

        let paths = get_paths(&config).await?;
        if paths
            .into_iter()
            .any(|path| request_path.starts_with(&path))
        {
            // Custom path
            addr = get_forward_by_path(&config, request_path).await?;
            // TODO:
            // replace request_path in buf with /
        } else {
            // Default web path
            addr = get_forward_by_name(&config, "web").await?;
        }
    } else {
        tracing::debug!("Default redirect");
        addr = get_forward_by_name(&config, "web").await?;
    }
    redirect(incoming, &addr, buf).await?;
    Ok(())
}

async fn redirect(mut incoming: TcpStream, addr: &str, buf: Vec<u8>) -> Result<()> {
    let mut outcoming = TcpStream::connect(addr).await?;
    outcoming
        .write_all(&buf)
        .await
        .context(format!(" writing initial buffer {addr}"))?;

    let (mut incoming_read, mut incoming_write) = incoming.split();
    let (mut outcoming_read, mut outcoming_write) = outcoming.split();

    let incoming_fut = io::copy(&mut incoming_read, &mut outcoming_write);
    let outcoming_fut = io::copy(&mut outcoming_read, &mut incoming_write);

    try_join!(incoming_fut, outcoming_fut)?;
    Ok(())
}
