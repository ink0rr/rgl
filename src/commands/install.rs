use crate::info;
use crate::rgl::{FilterInstaller, FilterType};
use anyhow::Result;

pub fn install_filters(filters: Vec<&String>, force: bool) -> Result<()> {
    for arg in filters {
        info!("Installing filter <b>{}</>...", arg);
        let filter = FilterInstaller::from_arg(arg)?;
        if filter.install(FilterType::Global, None, force)? {
            info!("Filter <b>{}</> successfully installed", filter.name);
        }
    }
    Ok(())
}
