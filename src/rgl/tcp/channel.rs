use anyhow::{anyhow, Result};
use std::fmt::Display;
use tokio::sync::mpsc;

pub struct TcpChannel {
    sender: mpsc::Sender<String>,
    receiver: mpsc::Receiver<String>,
}

impl TcpChannel {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(32);
        TcpChannel { sender, receiver }
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
