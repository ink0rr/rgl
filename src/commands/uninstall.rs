use super::Command;
use crate::rgl::GlobalFilters;
use crate::{info, warn};
use anyhow::Result;
use clap::Args;

/// Uninstall filter(s)
#[derive(Args)]
pub struct Uninstall {
    #[arg(required = true)]
    filters: Vec<String>,
}

impl Command for Uninstall {
    fn dispatch(&self) -> Result<()> {
        let mut global_filters = GlobalFilters::load()?;
        for name in &self.filters {
            if global_filters.remove(name).is_some() {
                info!("Uninstalled filter <b>{name}</>");
            } else {
                warn!("Filter <b>{name}</> not found");
            }
        }
        global_filters.save()
    }
    fn error_context(&self) -> String {
        "Error uninstalling filter".to_owned()
    }
}
