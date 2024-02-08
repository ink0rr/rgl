use crate::fs::rimraf;
use crate::rgl::FilterType;
use crate::{info, warn};
use anyhow::Result;

pub fn uninstall_tools(tools: Vec<&String>) -> Result<()> {
    for name in tools {
        let filter_dir = FilterType::Tool.cache_dir(name)?;
        if filter_dir.exists() {
            rimraf(filter_dir)?;
            info!("Uninstalled tool <b>{name}</>");
        } else {
            warn!("Tool <b>{name}</> not found");
        }
    }
    Ok(())
}
