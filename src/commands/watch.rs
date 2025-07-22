use super::Command;
use crate::rgl::{runner, Config, MinecraftServer, Session, UserConfig};
use crate::{error, info, log, warn};
use anyhow::Result;
use clap::Args;

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
                let mut session = Session::lock()?;

                if let Err(e) = runner(&config, &self.profile, self.clean, compat) {
                    error!("{}", self.error_context());
                    e.chain().for_each(|e| log!("<red>[+]</> {e}"));
                }

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
                config.watch_project_files()?;
                warn!("Changes detected, restarting...");
                session.unlock()?;
            }
        })
    }
    fn error_context(&self) -> String {
        format!("Error running <b>{}</> profile", self.profile)
    }
}
