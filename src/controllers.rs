use std::sync::Arc;

use anyhow::{anyhow, Result};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    try_join,
};

use crate::{
    config::{init_config, Config},
    utils::{get_forward_by_name, get_forward_by_path, init_tracing},
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
    tracing::info!("Got request -> {}", request);

    if request.starts_with("SSH-") {
        tracing::info!("Redirect to ssh");
        addr = get_forward_by_name(&config, "ssh").await?;
    } else if request.starts_with("GET") {
        let request_path = request.split_whitespace().nth(1).unwrap_or("/");
        tracing::info!("Redirect to web with path {request_path}");
        addr = get_forward_by_path(&config, &request_path).await?;
        // TODO !!!
        // replace custom path like /cv in stream (buf) to /
    } else {
        tracing::info!("Default redirect");
        addr = get_forward_by_path(&config, "/").await?;
    }
    redirect(incoming, &addr, buf).await?;
    Ok(())
}

async fn redirect(mut incoming: TcpStream, addr: &str, buf: Vec<u8>) -> Result<()> {
    let mut outcoming = TcpStream::connect(addr).await?;
    let (mut incoming_read, mut incoming_write) = incoming.split();
    let (mut outcoming_read, mut outcoming_write) = outcoming.split();

    outcoming_write.write_all(&buf).await?;

    let incoming_fut = io::copy(&mut incoming_read, &mut outcoming_write);
    let outcoming_fut = io::copy(&mut outcoming_read, &mut incoming_write);

    try_join!(incoming_fut, outcoming_fut)?;
    Ok(())
}
