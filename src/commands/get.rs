use super::Command;
use crate::info;
use crate::rgl::{
    Config, Filter, FilterContext, FilterDefinition, FilterInstaller, FilterType, Session,
};
use anyhow::Result;
use clap::Args;
use semver::Version;

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
        for (name, value) in config.get_filters() {
            match FilterDefinition::from_value(value)? {
                FilterDefinition::Local(filter) => {
                    info!("Installing dependencies for <b>{name}</>...");
                    let context = FilterContext::new(FilterType::Local, &name)?;
                    filter.install_dependencies(&context)?;
                }
                FilterDefinition::Remote(filter) => {
                    info!("Getting filter <b>{name}</>...");
                    let git_ref = Version::parse(&filter.version)
                        .map(|version| format!("{name}-{version}"))
                        .unwrap_or(filter.version);
                    let filter = FilterInstaller::new(&name, filter.url, git_ref);
                    filter.install(FilterType::Remote, Some(&data_path), self.force)?;
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
