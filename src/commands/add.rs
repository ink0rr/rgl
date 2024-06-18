use super::Command;
use crate::fs::copy_dir;
use crate::info;
use crate::rgl::{get_filter_cache_dir, Config, RemoteFilter, Session};
use anyhow::Result;
use clap::Args;

/// Add filter(s) to current project
#[derive(Args)]
pub struct Add {
    #[arg(required = true)]
    filters: Vec<String>,
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
            remote.install(&name, self.force)?;
            info!("Filter <b>{name}</> successfully added");

            let filter_data = get_filter_cache_dir(&name, &remote)?.join("data");
            let target_path = data_path.join(&name);
            if filter_data.is_dir() && !target_path.exists() {
                info!("Copying filter data to <b>{}</>", target_path.display());
                copy_dir(filter_data, target_path)?;
            }
            config.add_filter(&name, &remote.into())?;
        }
        config.save()?;
        session.unlock()
    }
    fn error_context(&self) -> String {
        "Error adding filter".to_owned()
    }
}
