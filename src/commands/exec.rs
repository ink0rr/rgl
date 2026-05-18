use super::Command;
use crate::fs::{copy_dir, empty_dir, sync_dir};
use crate::info;
use crate::rgl::{Config, Filter, FilterContext, GlobalFilters, Session, Temp};
use anyhow::Result;
use clap::Args;

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
        let data = config.get_data_path();

        let temp = Temp::from_dot_regolith();

        empty_dir(&temp.root)?;
        if let Some(bp) = &bp {
            copy_dir(bp, &temp.bp)?;
        }
        if let Some(rp) = &rp {
            copy_dir(rp, &temp.rp)?;
        }
        copy_dir(&data, &temp.data)?;

        if let Ok(filter) = config.get_filter(&self.filter) {
            info!("Running filter <filter>{}</>", self.filter);
            let context = FilterContext::new(&self.filter, &filter)?;
            filter.run(&context, &temp.root, &self.run_args)?;
        } else {
            let global_filters = GlobalFilters::load()?;
            let filter = global_filters.get(&self.filter)?.into();
            info!("Running global filter <filter>{}</>", self.filter);
            let context = FilterContext::new(&self.filter, &filter)?;
            filter.run(&context, &temp.root, &self.run_args)?;
        }

        info!("Applying changes to source directory:");
        if let Some(bp) = bp {
            println!("\tBP: {}", bp.display());
            sync_dir(temp.bp, bp)?;
        }
        if let Some(rp) = rp {
            println!("\tRP: {}", rp.display());
            sync_dir(temp.rp, rp)?;
        }
        sync_dir(temp.data, data)?;

        info!("Successfully executed filter <filter>{}</>", self.filter);
        session.unlock()
    }
    fn error_context(&self) -> String {
        format!("Error executing filter <filter>{}</>", self.filter)
    }
}
