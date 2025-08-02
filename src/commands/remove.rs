use super::Command;
use crate::rgl::{Config, ConfigCst, Session};
use crate::{info, warn};
use anyhow::Result;
use clap::Args;

/// Remove filter(s) from current project
#[derive(Args)]
#[clap(alias = "rm")]
pub struct Remove {
    #[arg(required = true)]
    filters: Vec<String>,
}

impl Command for Remove {
    fn dispatch(&self) -> Result<()> {
        // Make sure it's a valid config
        let _ = Config::load()?;
        let config_cst = ConfigCst::load()?;
        let mut session = Session::lock()?;
        for name in &self.filters {
            if config_cst.remove_filter(name) {
                info!("Removed filter <b>{name}</>");
            } else {
                warn!("Filter <b>{name}</> not found");
            }
        }
        config_cst.save()?;
        session.unlock()
    }
    fn error_context(&self) -> String {
        "Error removing filter".to_owned()
    }
}
