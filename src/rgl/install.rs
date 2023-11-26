use super::{
    ref_to_version, Config, Filter, FilterContext, FilterDefinition, FilterInstaller, RemoteFilter,
    Session,
};
use crate::info;
use anyhow::Result;
use semver::Version;
use std::path::Path;

pub fn install_filters(force: bool) -> Result<()> {
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
                info!("Installing filter <b>{name}</>...");
                let git_ref = Version::parse(&filter.version)
                    .map(|version| format!("{name}-{version}"))
                    .unwrap_or(filter.version);
                let filter = FilterInstaller::new(&name, filter.url, git_ref)?;
                filter.install(data_path, force)?;
            }
        };
    }
    info!("Successfully installed all filters");
    session.unlock()
}

pub fn install_add(filters: Vec<&String>, force: bool) -> Result<()> {
    let mut config = Config::load()?;
    let mut session = Session::lock()?;
    let data_path = Path::new(&config.regolith.data_path);
    for arg in filters {
        info!("Installing filter <b>{}</>...", arg);
        let filter = FilterInstaller::from_arg(arg)?;
        if filter.install(data_path, force)? {
            info!("Filter <b>{}</> successfully installed", filter.name);
            let version = ref_to_version(&filter.git_ref);
            config.regolith.filter_definitions.insert(
                filter.name,
                serde_json::to_value(RemoteFilter {
                    url: filter.url,
                    version,
                })?,
            );
        }
    }
    config.save()?;
    session.unlock()
}
