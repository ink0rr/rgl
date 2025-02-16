use super::{TcpChannel, TcpProxy, TcpServer};
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use std::sync::Arc;
use tokio::sync::Mutex;

#[enum_dispatch]
pub enum TcpServerType {
    Proxy(TcpProxy),
    Server(TcpServer),
}

#[enum_dispatch(TcpServerType)]
pub trait TcpTrait {
    async fn serve(&mut self) -> Result<()>;
    async fn wait_for_client(&mut self, channel: Arc<Mutex<TcpChannel>>) -> Result<()>;
}
