use super::Command;
use crate::info;
use crate::rgl::{Config, Filter, FilterContext, FilterDefinition, Session};
use anyhow::Result;
use clap::Args;

/// Fetch filters defined in the `config.json` file
#[derive(Args)]
pub struct Get {
    #[arg(short, long)]
    force: bool,
}

impl Command for Get {
    fn dispatch(&self) -> Result<()> {
        let config = Config::load()?;
        let mut session = Session::lock()?;
        let data_path = config.get_data_path();
        for (name, filter) in config.get_filters()? {
            match filter {
                FilterDefinition::Remote(remote) => {
                    info!("Downloading filter <filter>{name}</>...");
                    remote.install(&name, Some(&data_path), false)?;
                }
                filter => {
                    info!("Installing dependencies for <filter>{name}</>...");
                    let context = FilterContext::new(&name, &filter)?;
                    filter.install_dependencies(&context)?;
                }
            };
        }
        info!("Success getting filters!");
        session.unlock()
    }
    fn error_context(&self) -> String {
        "Error getting filters".to_owned()
    }
}
