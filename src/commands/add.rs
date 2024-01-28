use crate::info;
use crate::rgl::{ref_to_version, Config, FilterInstaller, RemoteFilter, Session};
use anyhow::Result;
use std::path::Path;

pub fn add_filters(filters: Vec<&String>, force: bool) -> Result<()> {
    let mut config = Config::load()?;
    let mut session = Session::lock()?;
    let data_path = Path::new(&config.regolith.data_path);
    for arg in filters {
        info!("Adding filter <b>{}</>...", arg);
        let filter = FilterInstaller::from_arg(arg)?;
        if filter.install(data_path, force)? {
            info!("Filter <b>{}</> successfully added", filter.name);
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
