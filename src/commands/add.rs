use crate::info;
use crate::rgl::{Config, FilterInstaller, FilterType, Session};
use anyhow::Result;

pub fn add_filters(filters: Vec<&String>, force: bool) -> Result<()> {
    let mut config = Config::load()?;
    let mut session = Session::lock()?;
    let data_path = config.get_data_path();
    for arg in filters {
        info!("Adding filter <b>{}</>...", arg);
        let filter = FilterInstaller::from_arg(arg)?;
        if filter.install(FilterType::Remote, Some(&data_path), force)? {
            info!("Filter <b>{}</> successfully added", filter.name);
            config.add_filter(filter)?;
        }
    }
    config.save()?;
    session.unlock()
}
