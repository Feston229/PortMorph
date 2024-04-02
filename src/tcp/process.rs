use anyhow::{anyhow, Result};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

use crate::{
    config::ConfigInner,
    tcp::tunnel,
    utils::{find_path, get_forward_by_name, get_forward_by_path},
};

// Processes TCP connections by reading the incoming request and forwarding it to the appropriate service.
pub async fn process_tcp<S>(mut incoming: S, config: Arc<ConfigInner>) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Sized,
{
    // Retrieve the forwarding address for SSH connections from the configuration.
    let ssh_location = get_forward_by_name(&config, "ssh").await?;

    // Buffer to store the incoming request data.
    let mut buf: Vec<u8> = vec![];
    // Read the incoming request into the buffer.
    incoming.read_buf(&mut buf).await?;

    // Convert the request buffer into a string for processing.
    let request = String::from_utf8_lossy(&buf).to_string();
    tracing::debug!("Got request -> {}", request);

    // Forward the request based on its type (SSH or HTTP) or return an error for unknown requests.
    if request.contains("SSH") && !ssh_location.is_empty() {
        tunnel(incoming, ssh_location, &buf).await
    } else if request.contains("HTTP") {
        process_http(incoming, config, request, buf).await
    } else {
        Err(anyhow!("Unknown request"))
    }
}

// Processes HTTP requests by determining the forwarding address and modifying the request as necessary.
async fn process_http<S>(
    incoming: S,
    config: Arc<ConfigInner>,
    mut initial_request: String,
    mut buf: Vec<u8>,
) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Sized,
{
    // Address to forward the request to.
    let addr: String;
    // Extract the HTTP method from the request.
    let method = initial_request
        .split_whitespace()
        .nth(0)
        .ok_or(anyhow!("Unknown method"))?;
    // Extract the request path from the request.
    let request_path = initial_request
        .split_whitespace()
        .nth(1)
        .ok_or(anyhow!("Missing path"))?;

    // Find the internal path based on the request path and method.
    let path = find_path(&config, request_path, method).await?;
    // Determine the forwarding address and modify the request if necessary.
    if path != "/" {
        addr = get_forward_by_path(&config, &path).await?;
        // Modify the initial request to remove the internal path.
        initial_request = initial_request.replace(&format!(" {path}"), " ").into();
        // Update the request buffer with the modified request.
        buf = initial_request.as_bytes().to_vec();
    } else {
        // Use the default web forwarding address if the path is root.
        addr = get_forward_by_name(&config, "web").await?;
    }

    // Forward the modified request to the determined address.
    tunnel(incoming, addr, &buf).await
}
