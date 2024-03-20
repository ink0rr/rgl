use crate::fs::{empty_dir, try_symlink};
use crate::info;
use crate::rgl::{Config, Filter, FilterContext, Session};
use anyhow::Result;
use std::path::Path;

pub fn apply(filter_name: &str, run_args: Vec<String>) -> Result<()> {
    let config = Config::load()?;
    let mut session = Session::lock()?;

    let temp = Path::new(".regolith").join("tmp");
    let temp_bp = temp.join("BP");
    let temp_rp = temp.join("RP");

    empty_dir(&temp)?;
    try_symlink(config.get_behavior_pack(), temp_bp)?;
    try_symlink(config.get_resource_pack(), temp_rp)?;
    try_symlink(config.get_data_path(), temp.join("data"))?;

    let filter = config.get_filter(filter_name)?;
    let context = FilterContext::new(filter.get_type(), filter_name)?;
    filter.run(&context, &temp, &run_args)?;
    info!("Successfully applied filter <b>{filter_name}</>");

    session.unlock()
}
