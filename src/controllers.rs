use std::{fs::File, io::BufReader, sync::Arc};

use anyhow::{anyhow, Result};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    try_join,
};
use tokio_rustls::TlsAcceptor;

use crate::{
    config::{init_config, Config},
    utils::{find_path, get_forward_by_name, get_forward_by_path, init_tracing, is_ssl_enabled},
};

pub async fn run() -> Result<()> {
    init_tracing()?;
    let config = Arc::new(Mutex::new(init_config().await?));

    if is_ssl_enabled(&config).await {
        tls_listener(config).await?;
    } else {
        tcp_listener(config).await?;
    }

    Ok(())
}

async fn tls_listener(config: Arc<Mutex<Config>>) -> Result<()> {
    let config_lock = config.lock().await;
    let cert_path = config_lock
        .server
        .cert_path
        .as_ref()
        .ok_or(anyhow!("Missing cert_path"))?;
    let key_path = config_lock
        .server
        .key_path
        .as_ref()
        .ok_or(anyhow!("Missing key_path"))?;

    let certs = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(cert_path)?))
        .collect::<Result<Vec<_>, _>>()?;
    let private_key = rustls_pemfile::private_key(&mut BufReader::new(&mut File::open(key_path)?))?
        .ok_or(anyhow!("Incorrect key"))?;
    let server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)?;

    let acceptor = TlsAcceptor::from(Arc::new(server_config));
    let listener = TcpListener::bind(&config_lock.server.listen).await?;
    tracing::info!("Listening on {} (tls)", listener.local_addr()?);

    while let Ok((incoming, _)) = listener.accept().await {
        let mut buf: Vec<u8> = vec![];
        let mut incoming = acceptor.accept(incoming).await?;
        incoming.read_buf(&mut buf).await?;
        let request = String::from_utf8_lossy(&buf);
        tracing::debug!("Got request -> {} (tls)", request);
    }

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
