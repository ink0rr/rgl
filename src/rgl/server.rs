use crate::log;
use anyhow::Result;
use async_tungstenite::{accept_async, WebSocketSender};
use dashmap::DashMap;
use serde_json::json;
use smol::{
    net::{TcpListener, TcpStream},
    stream::StreamExt,
};
use std::{net::SocketAddr, sync::Arc};
use uuid::Uuid;

pub struct MinecraftServer {
    peers: Arc<DashMap<SocketAddr, WebSocketSender<TcpStream>>>,
}

impl MinecraftServer {
    pub fn bind_and_accept(port: u16) -> Result<Self> {
        let peers = Arc::new(DashMap::new());
        let listener = smol::block_on(TcpListener::bind(format!("127.0.0.1:{}", port)))?;
        let addr = listener.local_addr()?;
        let port = addr.port();
        log!("<green>[SERVER]</> WebSocket is running at port {port}. Run <bright-yellow>/connect {addr}</> in-game to automatically reload scripts");

        smol::spawn({
            let peers = peers.clone();
            async move {
                loop {
                    let (stream, addr) = match listener.accept().await {
                        Ok(v) => v,
                        Err(e) => {
                            log!("<red>[SERVER]</> {e}");
                            continue;
                        }
                    };

                    let ws = match accept_async(stream).await {
                        Ok(v) => v,
                        Err(e) => {
                            log!("<red>[SERVER]</> {e}");
                            continue;
                        }
                    };
                    log!("<green>[SERVER]</> Client {addr} connected");

                    let (tx, mut rx) = ws.split();
                    peers.insert(addr, tx);

                    smol::spawn({
                        let peers = peers.clone();
                        async move {
                            while let Some(msg) = rx.next().await {
                                if let Err(e) = msg {
                                    log!("<red>[SERVER]</> {e} (client {addr})");
                                    break;
                                }
                            }
                            log!("<yellow>[SERVER]</> Client {addr} disconnected");
                            peers.remove(&addr)
                        }
                    })
                    .detach();
                }
            }
        })
        .detach();

        Ok(Self { peers })
    }

    pub async fn run_command(&self, cmd: &str) {
        let request = json!({
            "header": {
                "version": 1,
                "requestId": Uuid::new_v4().to_string(),
                "messageType": "commandRequest",
                "messagePurpose": "commandRequest"
            },
            "body": {
                "version": 1,
                "origin": { "type": "player" },
                "commandLine": cmd,
            }
        });
        for entry in self.peers.iter() {
            let (addr, tx) = entry.pair();
            if let Err(e) = tx.send(request.to_string().into()).await {
                log!("<red>[SERVER]</> Failed to send message: {e} (client {addr})");
            }
        }
    }
}
