use crate::{info, warn};
use anyhow::{anyhow, bail, Result};
use std::fmt::Display;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};

pub struct ProxyChannel {
    sender: mpsc::Sender<String>,
    receiver: mpsc::Receiver<String>,
}

pub struct Proxy {
    listen_addr: String,
    server_addr: String,
    listener: Option<TcpListener>,
}

impl ProxyChannel {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(32);
        ProxyChannel { sender, receiver }
    }
    pub async fn recv(&mut self) -> Result<String> {
        self.receiver
            .recv()
            .await
            .ok_or_else(|| anyhow!("Channel closed"))
    }
    pub async fn send_command<S: Into<String> + Display>(&mut self, command: S) -> Result<()> {
        let message = format!(
            r#"{{"type":"minecraftCommand","command":{{"command":"{}","dimension_type":"overworld"}}}}"#,
            command
        );
        let message_len = message.len() + 1;
        let header = format!("{:08x}", message_len);
        let data = format!("{}\n{}\n", header, message);
        self.sender.send(data).await?;
        Ok(())
    }
}

impl Proxy {
    pub fn new<S: Into<String>>(listen_addr: S, server_addr: S) -> Self {
        Proxy {
            listen_addr: listen_addr.into(),
            server_addr: server_addr.into(),
            listener: None,
        }
    }
    pub async fn serve(&mut self) -> Result<()> {
        let listener = TcpListener::bind(&self.listen_addr).await?;
        info!("Listening on {}", self.listen_addr);
        self.listener = Some(listener);
        Ok(())
    }
    pub async fn wait_for_client(&mut self, proxy_channel: Arc<Mutex<ProxyChannel>>) -> Result<()> {
        if self.listener.is_none() {
            bail!("Listener is not initialized");
        }
        let listener = self.listener.as_ref().unwrap();
        loop {
            let (inbound, _) = listener.accept().await?;
            let peer_addr = inbound.peer_addr()?;
            info!("New connection: {}", peer_addr.clone());
            Self::handle_connection(inbound, &self.server_addr, proxy_channel.clone()).await?;
            warn!("Connection closed: {}", peer_addr);
        }
    }
    async fn handle_connection(
        mut inbound: TcpStream,
        target_addr: &str,
        proxy_channel: Arc<Mutex<ProxyChannel>>,
    ) -> Result<()> {
        let mut outbound = TcpStream::connect(&target_addr).await?;
        let (mut ri, mut wi) = inbound.split();
        let (mut ro, mut wo) = outbound.split();
        let mut handshake = false;
        loop {
            let client_to_server = Self::send_packet(&mut ri, &mut wo);
            let server_to_client = Self::send_packet(&mut ro, &mut wi);
            let mut proxy_channel = proxy_channel.lock().await;
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
                data = proxy_channel.recv() => {
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
        let mut buffer = [0; 1024];
        let n = reader.read(&mut buffer).await?;
        if n == 0 {
            bail!("EOF");
        }
        let data = &buffer[..n];
        writer.write_all(data).await?;
        if data.starts_with(b"00000012") {
            return Ok(true);
        }
        Ok(false)
    }
}
