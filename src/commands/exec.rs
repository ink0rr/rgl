use super::Command;
use crate::fs::{empty_dir, read_json, try_symlink};
use crate::info;
use crate::rgl::{Config, Filter, FilterContext, FilterType, RemoteFilterConfig, Session};
use anyhow::{Context, Result};
use clap::Args;
use std::path::Path;

/// Executes a filter and apply changes to the current project
#[derive(Args)]
#[clap(alias = "x")]
pub struct Exec {
    filter: String,
    run_args: Vec<String>,
}

impl Command for Exec {
    fn dispatch(&self) -> Result<()> {
        let config = Config::load()?;
        let mut session = Session::lock()?;

        let temp = Path::new(".regolith").join("tmp");
        let temp_bp = temp.join("BP");
        let temp_rp = temp.join("RP");

        empty_dir(&temp)?;
        try_symlink(config.get_behavior_pack(), temp_bp)?;
        try_symlink(config.get_resource_pack(), temp_rp)?;
        try_symlink(config.get_data_path(), temp.join("data"))?;

        if let Ok(filter) = config.get_filter(&self.filter) {
            info!("Running local filter <b>{}</>", self.filter);
            let context = FilterContext::new(filter.get_type(), &self.filter)?;
            filter.run(&context, &temp, &self.run_args)?;
        } else {
            info!("Running global filter <b>{}</>", self.filter);
            let context = FilterContext::new(FilterType::Global, &self.filter)?;
            let config_path = context.filter_dir.join("filter.json");
            let config = read_json::<RemoteFilterConfig>(config_path).context(format!(
                "Failed to load config for filter <b>{}</>",
                self.filter
            ))?;
            for filter in config.filters {
                filter.run(&context, &temp, &self.run_args)?;
            }
        }
        session.unlock()
    }
    fn error_context(&self) -> String {
        format!("Error executing filter <b>{}</>", self.filter)
    }
}
