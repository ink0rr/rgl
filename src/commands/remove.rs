use crate::fs::rimraf;
use crate::rgl::{Config, FilterType, Session};
use crate::{info, warn};
use anyhow::Result;

pub fn remove_filters(filters: Vec<&String>) -> Result<()> {
    let mut config = Config::load()?;
    let mut session = Session::lock()?;
    for name in filters {
        if config.remove_filter(name).is_some() {
            let filter_dir = FilterType::Remote.cache_dir(name)?;
            rimraf(filter_dir)?;
            info!("Removed filter <b>{name}</>");
        } else {
            warn!("Filter <b>{name}</> not found");
        }
    }
    config.save()?;
    session.unlock()
}
