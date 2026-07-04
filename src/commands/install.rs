use super::Command;
use crate::info;
use crate::rgl::{GlobalFilters, RemoteFilter};
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
    /// Force resolvers refresh
    #[arg(long)]
    force_resolver_refresh: bool,
}

impl Command for Install {
    fn dispatch(&self) -> Result<()> {
        let mut global_filters = GlobalFilters::load()?;
        for arg in &self.filters {
            info!("Installing filter <filter>{arg}</>...");
            let (name, remote) = RemoteFilter::parse(arg, self.force_resolver_refresh)?;
            remote.install(&name, None, self.force)?;

            info!("Filter <filter>{name}</> successfully installed");
            global_filters.add(&name, remote);
        }
        global_filters.save()
    }
    fn error_context(&self) -> String {
        "Error installing filter".to_owned()
    }
}
