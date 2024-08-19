use super::Command;
use crate::rgl::{Config, FilterDefinition, Session};
use crate::{info, warn};
use anyhow::{Context, Result};
use clap::Args;

/// Update filter(s) in the current project
#[derive(Args)]
pub struct Update {
    filters: Vec<String>,
    #[arg(short, long)]
    force: bool,
}

impl Command for Update {
    fn dispatch(&self) -> Result<()> {
        let mut config = Config::load()?;
        let mut session = Session::lock()?;
        let data_path = config.get_data_path();

        info!("Updating filters...");
        if self.filters.is_empty() {
            for (name, filter) in config.get_filters()? {
                if let FilterDefinition::Remote(mut remote) = filter {
                    remote
                        .update(&name, Some(&data_path), self.force)
                        .context(format!("Failed to update filter {name}"))?;
                    config.add_filter(&name, &remote.into())?;
                }
            }
        } else {
            for name in &self.filters {
                let filter = config.get_filter(name)?;
                if let FilterDefinition::Remote(mut remote) = filter {
                    remote
                        .update(name, Some(&data_path), self.force)
                        .context(format!("Failed to update filter {name}"))?;
                    config.add_filter(name, &remote.into())?;
                } else {
                    warn!("Filter <b>{name}</> is not a remote filter, skipping...");
                }
            }
        }
        info!("Filters successfully updated");
        config.save()?;
        session.unlock()
    }
    fn error_context(&self) -> String {
        "Error updating filter".to_owned()
    }
}
