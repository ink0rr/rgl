use super::{TcpChannel, TcpTrait};
use crate::{info, warn};
use anyhow::{bail, Result};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub struct TcpProxy {
    listen_addr: String,
    server_addr: String,
    listener: Option<TcpListener>,
}

impl TcpTrait for TcpProxy {
    async fn serve(&mut self) -> Result<()> {
        let listener = TcpListener::bind(&self.listen_addr).await?;
        info!("Listening on {}", self.listen_addr);
        self.listener = Some(listener);
        Ok(())
    }
    async fn wait_for_client(
        &mut self,
        channel: Arc<Mutex<TcpChannel>>,
        notifier: Arc<Mutex<bool>>,
    ) -> Result<()> {
        if self.listener.is_none() {
            bail!("Listener is not initialized");
        }
        let listener = self.listener.as_ref().unwrap();
        loop {
            let (inbound, _) = listener.accept().await?;
            let peer_addr = inbound.peer_addr()?;
            info!("New connection: {}", peer_addr.clone());
            *notifier.lock().await = true;
            Self::handle_connection(inbound, &self.server_addr, channel.clone()).await?;
            warn!("Connection closed: {}", peer_addr);
            *notifier.lock().await = false;
        }
    }
}

impl TcpProxy {
    pub fn new<S: Into<String>>(listen_addr: S, server_addr: S) -> Self {
        TcpProxy {
            listen_addr: listen_addr.into(),
            server_addr: server_addr.into(),
            listener: None,
        }
    }
    async fn handle_connection(
        mut inbound: TcpStream,
        target_addr: &str,
        channel: Arc<Mutex<TcpChannel>>,
    ) -> Result<()> {
        let mut outbound = TcpStream::connect(&target_addr).await?;
        let (mut ri, mut wi) = inbound.split();
        let (mut ro, mut wo) = outbound.split();
        let mut handshake = false;
        loop {
            let client_to_server = Self::send_packet(&mut ri, &mut wo);
            let server_to_client = Self::send_packet(&mut ro, &mut wi);
            let mut channel = channel.lock().await;
            tokio::select! {
                result = client_to_server => {
                    if result.is_err() {
                        warn!("Error in client to server: {}", result.unwrap_err());
                        break;
                    }
                }
                result = server_to_client => {
                    if let Ok(true) = result {
                        handshake = true;
                    } else if result.is_err() {
                        warn!("Error in server to client: {}", result.unwrap_err());
                        break;
                    }
                }
                data = channel.recv() => {
                    if let Ok(data) = data {
                        if handshake {
                            wi.write_all(data.as_bytes()).await?;
                        }
                    } else {
                        warn!("Error receiving data from proxy channel: {}", data.unwrap_err());
                    }
                }
            }
        }
        Ok(())
    }
    async fn send_packet(
        reader: &mut (impl AsyncReadExt + Unpin),
        writer: &mut (impl AsyncWriteExt + Unpin),
    ) -> Result<bool> {
        let mut buf = [0u8; 9];
        Self::read(reader, &mut buf).await?;
        let size = u32::from_str_radix(&String::from_utf8_lossy(&buf).trim(), 16)?;
        let mut buffer = vec![0u8; 9 + size as usize];
        buffer[..9].copy_from_slice(&buf);
        Self::read(reader, &mut buffer[9..]).await?;
        writer.write_all(&buffer).await?;
        Ok(buffer.starts_with(b"00000012"))
    }
    async fn read(reader: &mut (impl AsyncReadExt + Unpin), buffer: &mut [u8]) -> Result<usize> {
        let size = reader.read(buffer).await?;
        if size == 0 {
            bail!("EOF");
        }
        Ok(size)
    }
}
