use super::Command;
use crate::rgl::{watch, TcpProxy, TcpServer, TcpServerType, UserConfig};
use anyhow::Result;
use clap::Args;

/// Watch for file changes and restart automatically
#[derive(Args)]
pub struct Watch {
    #[arg(default_value = "default")]
    profile: String,
    /// Do not use previous run output as cache
    #[arg(long)]
    no_cache: bool,
    /// Start a proxy server to be used with the Minecraft Debugger
    #[arg(long, conflicts_with = "server")]
    proxy: bool,
    /// Start a server for triggering automatic reload
    #[arg(long, conflicts_with = "proxy")]
    server: bool,
}

impl Command for Watch {
    fn dispatch(&self) -> Result<()> {
        let server_type = if self.proxy {
            let config = UserConfig::proxy_server();
            Some(TcpServerType::Proxy(TcpProxy::new(
                &config.listen_address,
                &config.server_address,
            )))
        } else if self.server {
            let config = UserConfig::proxy_server();
            Some(TcpServerType::Server(TcpServer::new(
                &config.listen_address,
            )))
        } else {
            None
        };
        watch(&self.profile, !self.no_cache, server_type)
    }
    fn error_context(&self) -> String {
        format!("Error running <b>{}</> profile", self.profile)
    }
}
