use super::Command;
use crate::info;
use crate::rgl::{Config, FilterInstaller, FilterType, Session};
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
            let filter = FilterInstaller::from_arg(arg)?;
            if filter.install(FilterType::Remote, Some(&data_path), self.force)? {
                info!("Filter <b>{}</> successfully added", filter.name);
                config.add_filter(filter)?;
            }
        }
        config.save()?;
        session.unlock()
    }
    fn error_context(&self) -> String {
        "Error adding filter".to_owned()
    }
}
