use super::Command;
use crate::fs::rimraf;
use crate::rgl::FilterType;
use crate::{info, warn};
use anyhow::Result;
use clap::Args;

/// Uninstall globally installed filter(s)
#[derive(Args)]
pub struct Uninstall {
    #[arg(required = true)]
    filters: Vec<String>,
}

impl Command for Uninstall {
    fn dispatch(&self) -> Result<()> {
        for name in &self.filters {
            let filter_dir = FilterType::Global.cache_dir(name)?;
            if filter_dir.exists() {
                rimraf(filter_dir)?;
                info!("Uninstalled filter <b>{name}</>");
            } else {
                warn!("Filter <b>{name}</> not found");
            }
        }
        Ok(())
    }
    fn error_context(&self) -> String {
        "Error uninstalling filter".to_owned()
    }
}
