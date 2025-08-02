use super::Command;
use crate::rgl::{Config, ConfigCst, FilterDefinition, Session};
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
        let config = Config::load()?;
        let config_cst = ConfigCst::load()?;
        let mut session = Session::lock()?;
        let data_path = config.get_data_path();

        info!("Updating filters...");
        if self.filters.is_empty() {
            for (name, definition) in config.get_filters()? {
                if let FilterDefinition::Remote(mut remote) = definition {
                    remote
                        .update(&name, Some(&data_path), self.force)
                        .context(format!("Failed to update filter {name}"))?;
                    config_cst.add_filter(&name, remote);
                }
            }
        } else {
            for name in &self.filters {
                let definition = config.get_filter(name)?;
                if let FilterDefinition::Remote(mut remote) = definition {
                    remote
                        .update(name, Some(&data_path), self.force)
                        .context(format!("Failed to update filter {name}"))?;
                    config_cst.add_filter(name, remote);
                } else {
                    warn!("Filter <b>{name}</> is not a remote filter, skipping...");
                }
            }
        }

        info!("Filters successfully updated");
        config_cst.save()?;
        session.unlock()
    }
    fn error_context(&self) -> String {
        "Error updating filter".to_owned()
    }
}
