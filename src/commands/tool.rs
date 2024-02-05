use crate::fs::{empty_dir, read_json, try_symlink};
use crate::rgl::{Config, Filter, FilterContext, FilterType, RemoteFilterConfig, Session};
use anyhow::{Context, Result};
use std::path::Path;

pub fn tool(tool_name: &str, run_args: Vec<String>) -> Result<()> {
    let config = Config::load()?;
    let mut session = Session::lock()?;

    let temp = Path::new(".regolith").join("tmp");
    let temp_bp = temp.join("BP");
    let temp_rp = temp.join("RP");

    empty_dir(&temp)?;
    try_symlink(config.packs.behavior_pack, temp_bp)?;
    try_symlink(config.packs.resource_pack, temp_rp)?;
    try_symlink(config.regolith.data_path, temp.join("data"))?;

    let context = FilterContext::new(FilterType::Tool, tool_name)?;
    let config_path = context.filter_dir.join("filter.json");
    let config = read_json::<RemoteFilterConfig>(config_path)
        .context(format!("Failed to load config for tool <b>{tool_name}</>"))?;

    for filter in config.filters {
        filter.run(&context, &temp, &run_args)?;
    }

    session.unlock()
}
