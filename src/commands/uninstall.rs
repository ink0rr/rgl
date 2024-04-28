use crate::fs::rimraf;
use crate::rgl::FilterType;
use crate::{info, warn};
use anyhow::Result;

pub fn uninstall_filters(filters: Vec<&String>) -> Result<()> {
    for name in filters {
        let filter_dir = FilterType::Global.cache_dir(name)?;
        if filter_dir.exists() {
            rimraf(filter_dir)?;
            info!("Uninstalled filter <b>{name}</>");
        } else {
            warn!("Filter <b>{name}</> not found");
        }
    }
    Ok(())
}
