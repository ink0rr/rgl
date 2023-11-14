use super::{ref_to_version, Config, FilterDefinition, FilterInstaller, RemoteFilterDefinition};
use crate::info;
use anyhow::Result;
use semver::Version;
use std::path::Path;

pub fn install_filters(force: bool) -> Result<()> {
    let config = Config::load()?;
    let data_path = Path::new(&config.regolith.data_path);
    for (name, def) in config.regolith.filter_definitions {
        if let FilterDefinition::Remote(def) = def {
            info!("Installing filter <b>{}</>...", name);
            let git_ref = Version::parse(&def.version)
                .map(|version| format!("{name}-{version}"))
                .unwrap_or(def.version);
            let filter = FilterInstaller::new(name, def.url, git_ref)?;
            filter.install(data_path, force)?;
        }
    }
    info!("Successfully installed all filters");
    Ok(())
}

pub fn install_add(filters: Vec<&String>, force: bool) -> Result<()> {
    let mut config = Config::load()?;
    let data_path = Path::new(&config.regolith.data_path);
    for arg in filters {
        info!("Installing filter <b>{}</>...", arg);
        let filter = FilterInstaller::from_arg(arg)?;
        if filter.install(data_path, force)? {
            info!("Filter <b>{}</> successfully installed", filter.name);
            let version = ref_to_version(&filter.git_ref);
            config.regolith.filter_definitions.insert(
                filter.name,
                FilterDefinition::Remote(RemoteFilterDefinition {
                    url: filter.url,
                    version,
                }),
            );
        }
    }
    config.save()
}
