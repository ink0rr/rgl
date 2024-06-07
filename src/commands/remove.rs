use super::Command;
use crate::fs::rimraf;
use crate::rgl::{Config, FilterType, Session};
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
        let mut config = Config::load()?;
        let mut session = Session::lock()?;
        for name in &self.filters {
            if config.remove_filter(name).is_some() {
                let filter_dir = FilterType::Remote.cache_dir(name)?;
                rimraf(filter_dir)?;
                info!("Removed filter <b>{name}</>");
            } else {
                warn!("Filter <b>{name}</> not found");
            }
        }
        config.save()?;
        session.unlock()
    }
    fn error_context(&self) -> String {
        "Error removing filter".to_owned()
    }
}
