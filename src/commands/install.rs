use super::Command;
use crate::info;
use crate::rgl::{FilterInstaller, FilterType};
use anyhow::Result;
use clap::Args;

/// Install filter(s) globally
#[derive(Args)]
#[clap(alias = "i")]
pub struct Install {
    #[arg(required = true)]
    filters: Vec<String>,
    #[arg(short, long)]
    force: bool,
}

impl Command for Install {
    fn dispatch(&self) -> Result<()> {
        for arg in &self.filters {
            info!("Installing filter <b>{}</>...", arg);
            let filter = FilterInstaller::from_arg(arg)?;
            if filter.install(FilterType::Global, None, self.force)? {
                info!("Filter <b>{}</> successfully installed", filter.name);
            }
        }
        Ok(())
    }
    fn error_context(&self) -> String {
        "Error installing filter".to_owned()
    }
}
