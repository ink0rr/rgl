use crate::rgl::{Config, Session};
use crate::{info, warn};
use anyhow::Result;

pub fn remove_filters(filters: Vec<&String>) -> Result<()> {
    let mut config = Config::load()?;
    let mut session = Session::lock()?;
    for name in filters {
        if config.regolith.filter_definitions.remove(name).is_some() {
            info!("Removed filter <b>{name}</>");
        } else {
            warn!("Filter <b>{name}</> not found");
        }
    }
    config.save()?;
    session.unlock()
}
