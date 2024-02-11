use anyhow::{anyhow, Result};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

use crate::{
    config::Config,
    tcp::tunnel,
    utils::{find_path, get_forward_by_name, get_forward_by_path},
};

pub async fn process_tcp<S>(mut incoming: S, config: Arc<Config>) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Sized,
{
    let mut buf: Vec<u8> = vec![];
    incoming.read_buf(&mut buf).await?;

    // Process request
    let request = String::from_utf8_lossy(&buf).to_string();
    tracing::debug!("Got request -> {}", request);

    if request.contains("SSH") {
        tunnel(incoming, get_forward_by_name(&config, "ssh").await?, &buf).await
    } else if request.contains("HTTP") {
        process_http(incoming, config, request, buf).await
    } else {
        Err(anyhow!("Unknown request"))
    }
}

async fn process_http<S>(
    incoming: S,
    config: Arc<Config>,
    mut initial_request: String,
    mut buf: Vec<u8>,
) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Sized,
{
    let addr: String;
    let method = initial_request
        .split_whitespace()
        .nth(0)
        .ok_or(anyhow!("Unknown method"))?;
    let request_path = initial_request
        .split_whitespace()
        .nth(1)
        .ok_or(anyhow!("Missing path"))?;

    let path = find_path(&config, request_path, method).await?;
    if path != "/" {
        addr = get_forward_by_path(&config, &path).await?;
        initial_request = initial_request.replace(&format!(" {path}"), " ").into();
        buf = initial_request.as_bytes().to_vec();
    } else {
        addr = get_forward_by_name(&config, "web").await?;
    }

    tunnel(incoming, addr, &buf).await
}
