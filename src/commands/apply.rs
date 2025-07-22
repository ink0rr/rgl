use super::Command;
use crate::fs::{copy_dir, empty_dir, sync_dir};
use crate::info;
use crate::rgl::{Config, Session, Temp};
use anyhow::Result;
use clap::Args;

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
        let data = config.get_data_path();

        let profile = config.get_profile(&self.profile)?;

        let temp = Temp::from_dot_regolith();

        empty_dir(&temp.root)?;
        copy_dir(&bp, &temp.bp)?;
        copy_dir(&rp, &temp.rp)?;
        copy_dir(&data, &temp.data)?;

        info!("Running <b>{}</> profile", self.profile);
        profile.run(&config, &temp.root, &self.profile)?;

        info!(
            "Applying changes to source directory: \n\
             \tBP: {} \n\
             \tRP: {}",
            bp.display(),
            rp.display()
        );
        sync_dir(temp.bp, bp)?;
        sync_dir(temp.rp, rp)?;
        sync_dir(temp.data, data)?;

        info!("Successfully applied profile <b>{}</>", self.profile);
        session.unlock()
    }
    fn error_context(&self) -> String {
        format!("Error applying profile <b>{}</>", self.profile)
    }
}
