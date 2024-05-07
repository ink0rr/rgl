use super::Command;
use crate::fs::{copy_dir, empty_dir, read_json, try_symlink};
use crate::info;
use crate::rgl::{
    copy_changed, Config, Filter, FilterContext, FilterType, RemoteFilterConfig, Session,
};
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

        let bp = config.get_behavior_pack();
        let rp = config.get_resource_pack();

        let temp = Path::new(".regolith").join("tmp");
        let temp_bp = temp.join("BP");
        let temp_rp = temp.join("RP");

        empty_dir(&temp)?;
        copy_dir(&bp, &temp_bp)?;
        copy_dir(&rp, &temp_rp)?;
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

        info!(
            "Applying changes to source directory: \n\
             \tBP: {} \n\
             \tRP: {}",
            bp.display(),
            rp.display()
        );
        copy_changed(temp_bp, bp)?;
        copy_changed(temp_rp, rp)?;

        info!("Successfully executed filter <b>{}</>", self.filter);
        session.unlock()
    }
    fn error_context(&self) -> String {
        format!("Error executing filter <b>{}</>", self.filter)
    }
}
