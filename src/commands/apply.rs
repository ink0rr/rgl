use super::Command;
use crate::fs::{copy_dir, empty_dir, sync_dir, try_symlink};
use crate::rgl::{Config, Session};
use crate::{info, measure_time};
use anyhow::Result;
use clap::Args;
use std::path::Path;

/// Runs a profile and apply changes to the current project
#[derive(Args)]
pub struct Apply {
    profile: String,
}

impl Command for Apply {
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

        let profile = config.get_profile(&self.profile)?;
        measure_time!(self.profile, {
            info!("Running <b>{}</> profile", self.profile);
            profile.run(&config, &temp, &self.profile)?;
        });

        info!(
            "Applying changes to source directory: \n\
             \tBP: {} \n\
             \tRP: {}",
            bp.display(),
            rp.display()
        );
        sync_dir(temp_bp, bp)?;
        sync_dir(temp_rp, rp)?;

        info!("Successfully applied profile <b>{}</>", self.profile);
        session.unlock()
    }
    fn error_context(&self) -> String {
        format!("Error applying profile <b>{}</>", self.profile)
    }
}
