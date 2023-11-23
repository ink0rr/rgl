use super::{ref_to_version, Config, FilterInstaller, RemoteFilter};
use crate::info;
use anyhow::Result;
use semver::Version;
use serde_json::Value;
use std::path::Path;

pub fn install_filters(force: bool) -> Result<()> {
    let config = Config::load()?;
    let data_path = Path::new(&config.regolith.data_path);
    for (name, def) in config.regolith.filter_definitions {
        let url = def["url"].to_owned();
        let version = def["version"].to_owned();
        if let (Value::String(url), Value::String(version)) = (url, version) {
            info!("Installing filter <b>{}</>...", name);
            let git_ref = Version::parse(&version)
                .map(|version| format!("{name}-{version}"))
                .unwrap_or(version);
            let filter = FilterInstaller::new(name, url, git_ref)?;
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
                serde_json::to_value(RemoteFilter {
                    url: filter.url,
                    version,
                })?,
            );
        }
    }
    config.save()
}
