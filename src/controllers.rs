use anyhow::Result;
use tokio::{
    io::{self, AsyncReadExt},
    net::{TcpListener, TcpStream},
    try_join,
};

use crate::utils::init_tracing;

pub async fn run() -> Result<()> {
    init_tracing()?;

    let listener = TcpListener::bind("127.0.0.1:40001").await?;
    tracing::info!("Listening on {}", listener.local_addr()?);

    while let Ok((incoming, _)) = listener.accept().await {
        tokio::spawn(async move {
            if let Err(e) = process_conn(incoming).await {
                tracing::error!("{e}");
            }
        });
    }

    Ok(())
}

async fn process_conn(mut incoming: TcpStream) -> Result<()> {
    let mut buf: Vec<u8> = vec![];

    if let Ok(_) = incoming.read_buf(&mut buf).await {
        let request = String::from_utf8_lossy(&buf);
        let request_lines: Vec<&str> = request.lines().collect();
        if let Some(identity) = request_lines.get(0) {
            if identity.starts_with("SSH-") {
                tracing::info!("Redirect to ssh");
                redirect(incoming, "127.0.0.1:40000").await?;
            }
        }
    }
    Ok(())
}

async fn redirect(mut incoming: TcpStream, addr: &str) -> Result<()> {
    if let Ok(mut outcoming) = TcpStream::connect(addr).await {
        let (mut incoming_read, mut incoming_write) = incoming.split();
        let (mut outcoming_read, mut outcoming_write) = outcoming.split();

        let incoming_fut = io::copy(&mut incoming_read, &mut outcoming_write);
        let outcoming_fut = io::copy(&mut outcoming_read, &mut incoming_write);

        try_join!(incoming_fut, outcoming_fut)?;
    }
    Ok(())
}
