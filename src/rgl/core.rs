use super::{copy_dir, empty_dir, move_dir, rimraf, symlink, Config, RunContext};
use crate::{info, measure_time, warn};
use anyhow::{Context, Result};
use std::{fs, io};

pub fn run_or_watch(profile_name: &str, watch: bool) -> Result<()> {
    let config = Config::load()?;

    let context = RunContext::new(config, profile_name);
    let profile = context.get_profile(profile_name)?;
    let temp = profile.get_temp_dir()?;
    let temp_bp = temp.join("BP");
    let temp_rp = temp.join("RP");
    let (bp, rp) = profile.get_export_paths(&context.name)?;

    measure_time!("Setup", {
        empty_dir(&temp)?;
        copy_dir(&context.behavior_pack, &temp_bp)?;
        copy_dir(&context.resource_pack, &temp_rp)?;
        if let Err(e) = symlink(&context.data_path, temp.join("data")) {
            match e.downcast_ref::<io::Error>().map(|e| e.kind()) {
                Some(io::ErrorKind::NotFound) => {
                    warn!("Data folder does not exists, creating...");
                    fs::create_dir_all(&context.data_path)?;
                }
                _ => return Err(e),
            }
        }
    });

    measure_time!(profile_name, {
        info!("Running <b>{profile_name}</> profile");
        profile.run(&context, &temp)?;
    });

    measure_time!("Export project", {
        info!(
            "Moving files to target location: \n\
            \tBP: {} \n\
            \tRP: {}",
            bp.display(),
            rp.display()
        );
        let export: Result<()> = {
            rimraf(&bp)?;
            rimraf(&rp)?;
            move_dir(temp_bp, bp)?;
            move_dir(temp_rp, rp)?;
            Ok(())
        };
        export.context("Failed to export project")?;
    });

    info!("Successfully ran the <b>{profile_name}</> profile");
    if watch {
        info!("Watching for changes...");
        info!("Press Ctrl+C to stop watching");
        context.watch_project_files()?;
        warn!("Changes detected, restarting...");
        return run_or_watch(profile_name, watch);
    }
    Ok(())
}
