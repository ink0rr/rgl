use super::Command;
use crate::rgl::{runner, Config, MinecraftServer, Session, UserConfig};
use crate::{error, info, log, warn};
use anyhow::Result;
use clap::Args;
use std::time::Duration;

/// Watch for file changes and restart automatically
#[derive(Args)]
pub struct Watch {
    #[arg(default_value = "default")]
    profile: String,
    /// Removes previous run output before running
    #[arg(long)]
    clean: bool,
    /// Enable this if filters are not working correctly
    #[arg(long)]
    compat: bool,
    /// Automatically reload scripts via WebSocket
    #[arg(long)]
    ws: bool,
}

impl Command for Watch {
    fn dispatch(&self) -> Result<()> {
        let compat = self.compat || UserConfig::force_compat();
        let server = if self.ws {
            Some(MinecraftServer::bind_and_accept(
                UserConfig::websocket_port(),
            )?)
        } else {
            None
        };

        smol::block_on(async {
            loop {
                let config = Config::load()?;
                let watcher = config.get_watcher()?;
                let mut session = Session::lock()?;

                let is_interrupted = smol::future::or(
                    async {
                        if let Err(e) = runner(&config, &self.profile, self.clean, compat).await {
                            error!("{}", self.error_context());
                            e.chain().for_each(|e| log!("<red>[+]</> {e}"));
                        }
                        false
                    },
                    async {
                        watcher.wait_changes().await;
                        true
                    },
                )
                .await;

                if !is_interrupted {
                    if let Some(server) = &server {
                        server.run_command("reload").await;
                        server
                            .run_command(
                                r#"tellraw @s {"rawtext": [{"translate": "commands.reload.success"}]}"#,
                            )
                            .await;
                    }

                    info!("Watching for changes...");
                    info!("Press Ctrl+C to stop watching");
                    watcher.flush();
                    watcher.wait_debounced(Duration::from_millis(100)).await;
                }

                warn!("Changes detected, restarting...");
                session.unlock()?;
            }
        })
    }
    fn error_context(&self) -> String {
        format!("Error running <profile>{}</> profile", self.profile)
    }
}
