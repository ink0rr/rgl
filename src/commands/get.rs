use crate::info;
use crate::rgl::{Config, Filter, FilterContext, FilterDefinition, FilterInstaller, Session};
use anyhow::Result;
use semver::Version;
use std::path::Path;

pub fn get_filters(force: bool) -> Result<()> {
    let config = Config::load()?;
    let mut session = Session::lock()?;
    let data_path = Path::new(&config.regolith.data_path);
    for (name, def) in config.regolith.filter_definitions {
        let filter = FilterDefinition::from_value(def)?;
        match filter {
            FilterDefinition::Local(filter) => {
                info!("Installing dependencies for <b>{name}</>...");
                let context = FilterContext::new(&name, false)?;
                filter.install_dependencies(&context)?;
            }
            FilterDefinition::Remote(filter) => {
                info!("Getting filter <b>{name}</>...");
                let git_ref = Version::parse(&filter.version)
                    .map(|version| format!("{name}-{version}"))
                    .unwrap_or(filter.version);
                let filter = FilterInstaller::new(&name, filter.url, git_ref);
                filter.install(data_path, force)?;
            }
        };
    }
    info!("Success getting filters!");
    session.unlock()
}
