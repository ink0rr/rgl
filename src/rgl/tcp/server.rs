use super::{PacketRequest, TcpChannel, TcpTrait};
use crate::{
    debug, info,
    rgl::{
        PacketEventType, PacketProtocol, PacketResponse, PacketResponseType, PacketResume,
        PacketStopOnException,
    },
    warn,
};
use anyhow::{bail, Result};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

pub struct TcpServer {
    listen_addr: String,
    listener: Option<TcpListener>,
}

impl TcpTrait for TcpServer {
    async fn serve(&mut self) -> Result<()> {
        let listener = TcpListener::bind(&self.listen_addr).await?;
        info!("Listening on {}", self.listen_addr);
        self.listener = Some(listener);
        Ok(())
    }
    async fn wait_for_client(&mut self, channel: Arc<Mutex<TcpChannel>>) -> Result<()> {
        let listener = match &self.listener {
            Some(listener) => listener,
            None => bail!("Listener is not initialized"),
        };
        loop {
            let (inbound, _) = listener.accept().await?;
            let peer_addr = inbound.peer_addr()?;
            info!("New connection: {}", peer_addr.clone());
            Self::handle_connection(inbound, channel.clone()).await?;
            warn!("Connection closed: {}", peer_addr);
        }
    }
}

impl TcpServer {
    pub fn new<S: Into<String>>(listen_addr: S) -> Self {
        TcpServer {
            listen_addr: listen_addr.into(),
            listener: None,
        }
    }
    async fn handle_connection(
        mut inbound: TcpStream,
        channel: Arc<Mutex<TcpChannel>>,
    ) -> Result<()> {
        let (mut r, mut w) = inbound.split();
        let mut handshake = false;
        loop {
            let handle_incoming = Self::handle_request(&mut r, &mut w);
            let mut channel = channel.lock().await;
            // TODO: Drop the read half of the channel once the handshake is complete?
            tokio::select! {
                result = handle_incoming => {
                    if result.is_err() {
                        warn!("Error in handle_request: {}", result.unwrap_err());
                        break;
                    } else {
                        handshake = true;
                    }
                }
                data = channel.recv() => {
                    if let Ok(data) = data {
                        if handshake {
                            w.write_all(data.as_bytes()).await?;
                        }
                    } else {
                        warn!("Error in channel: {}", data.unwrap_err());
                    }
                }
            }
        }
        Ok(())
    }
    async fn handle_request(
        reader: &mut (impl AsyncReadExt + Unpin),
        writer: &mut (impl AsyncWriteExt + Unpin),
    ) -> Result<()> {
        // 8 bytes header for size + 1 byte for newline
        let mut buf = [0u8; 9];
        Self::read(reader, &mut buf).await?;
        let size = u32::from_str_radix(&String::from_utf8_lossy(&buf).trim(), 16)?;
        let mut buf = vec![0u8; size as usize];
        Self::read(reader, &mut buf).await?;
        let packet: PacketRequest = match serde_json::from_slice(&buf) {
            Ok(packet) => packet,
            Err(e) => {
                debug!("Unable to parse packet: {}", e);
                return Ok(());
            }
        };
        Self::handle_packet(writer, packet).await?;
        Ok(())
    }
    async fn handle_packet(
        writer: &mut (impl AsyncWriteExt + Unpin),
        packet: PacketRequest,
    ) -> Result<()> {
        if let Some(event_type) = packet.event {
            match event_type {
                PacketEventType::ProtocolEvent(event) => {
                    if event.require_passcode {
                        // TODO: Implement passcode
                        bail!("Passcode required");
                    }
                    let plugin = event.plugins.first().unwrap();
                    let responses = [
                        PacketResponse {
                            packet_type: PacketResponseType::Protocol(PacketProtocol {
                                version: event.version,
                                target_module_uuid: plugin.module_uuid.clone(),
                            }),
                        },
                        PacketResponse {
                            packet_type: PacketResponseType::StopOnException(
                                PacketStopOnException {
                                    stop_on_exception: false,
                                },
                            ),
                        },
                        PacketResponse {
                            packet_type: PacketResponseType::Resume(PacketResume {}),
                        },
                    ];
                    for response in &responses {
                        response.write(writer).await?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    async fn read(reader: &mut (impl AsyncReadExt + Unpin), buffer: &mut [u8]) -> Result<usize> {
        let size = reader.read(buffer).await?;
        if size == 0 {
            bail!("EOF");
        }
        Ok(size)
    }
}
