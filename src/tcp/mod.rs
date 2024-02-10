pub mod listener;
mod process;

use anyhow::Result;
use tokio::{
    io::{self, AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};

pub async fn tunnel<S>(mut incoming: &mut S, addr: String, buf: &Vec<u8>) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    match TcpStream::connect(addr).await {
        // Redirect
        Ok(mut oncoming) => {
            oncoming.write_all(buf).await?;
            let (client_bytes, server_bytes) =
                io::copy_bidirectional(&mut incoming, &mut oncoming).await?;
            tracing::debug!(
                "client sent {client_bytes} bytes and server sent {server_bytes} bytes"
            );
        }
        // Return Bad Gateway html (TODO)
        Err(_) => {
            incoming.write_all(b"Bad Gateway").await?;
        }
    }
    Ok(())
}
