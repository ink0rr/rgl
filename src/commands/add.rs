use super::Command;
use crate::rgl::{Config, RemoteFilter, Session};
use crate::{info, warn};
use anyhow::Result;
use clap::Args;

/// Add filter(s) to current project
#[derive(Args)]
pub struct Add {
    #[arg(required = true)]
    filters: Vec<String>,
    #[arg(short, long, default_missing_value = "default", num_args = 0..)]
    profile: Vec<String>,
    #[arg(short, long)]
    force: bool,
}

impl Command for Add {
    fn dispatch(&self) -> Result<()> {
        let mut config = Config::load()?;
        let mut session = Session::lock()?;
        let data_path = config.get_data_path();
        for arg in &self.filters {
            info!("Adding filter <b>{}</>...", arg);
            let (name, remote) = RemoteFilter::parse(arg)?;
            remote.install(&name, Some(&data_path), self.force)?;
            for profile_name in &self.profile {
                if config.add_filter_to_profile(&name, profile_name) {
                    info!("Added filter <b>{name}</> to <b>{profile_name}</> profile");
                } else {
                    warn!("Profile <b>{profile_name}</> not found, skipping...");
                }
            }
            config.add_filter(&name, &remote.into())?;
            info!("Filter <b>{name}</> successfully added");
        }
        config.save()?;
        session.unlock()
    }
    fn error_context(&self) -> String {
        "Error adding filter".to_owned()
    }
}
