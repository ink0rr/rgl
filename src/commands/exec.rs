use crate::fs::{empty_dir, read_json, try_symlink};
use crate::info;
use crate::rgl::{Config, Filter, FilterContext, FilterType, RemoteFilterConfig, Session};
use anyhow::{Context, Result};
use std::path::Path;

pub fn exec(filter_name: &str, run_args: Vec<String>) -> Result<()> {
    let config = Config::load()?;
    let mut session = Session::lock()?;

    let temp = Path::new(".regolith").join("tmp");
    let temp_bp = temp.join("BP");
    let temp_rp = temp.join("RP");

    empty_dir(&temp)?;
    try_symlink(config.get_behavior_pack(), temp_bp)?;
    try_symlink(config.get_resource_pack(), temp_rp)?;
    try_symlink(config.get_data_path(), temp.join("data"))?;

    if let Ok(filter) = config.get_filter(filter_name) {
        info!("Running local filter <b>{filter_name}</>");
        let context = FilterContext::new(filter.get_type(), filter_name)?;
        filter.run(&context, &temp, &run_args)?;
    } else {
        info!("Running global filter <b>{filter_name}</>");
        let context = FilterContext::new(FilterType::Tool, filter_name)?;
        let config_path = context.filter_dir.join("filter.json");
        let config = read_json::<RemoteFilterConfig>(config_path).context(format!(
            "Failed to load config for tool <b>{filter_name}</>"
        ))?;

        for filter in config.filters {
            filter.run(&context, &temp, &run_args)?;
        }
    }

    session.unlock()
}
