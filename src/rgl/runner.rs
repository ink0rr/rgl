use super::{Config, ExportPaths, Session};
use crate::fs::{copy_dir, empty_dir, rimraf, symlink, sync_dir, try_symlink};
use crate::{info, measure_time, warn};
use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn run_or_watch(profile_name: &str, watch: bool, cached: bool) -> Result<()> {
    let config = Config::load()?;
    let mut session = Session::lock()?;

    let bp = config.get_behavior_pack();
    let rp = config.get_resource_pack();

    let profile = config.get_profile(profile_name)?;
    let (target_bp, target_rp) = profile
        .export
        .get_paths(config.get_name(), profile_name)
        .context("Failed to get export paths")?;

    let temp = PathBuf::from(".regolith").join("tmp");
    let temp_bp = temp.join("BP");
    let temp_rp = temp.join("RP");
    let temp_data = temp.join("data");

    measure_time!("Setup temp", {
        if cached {
            sync_dir(bp, &target_bp)?;
            sync_dir(rp, &target_rp)?;
        } else {
            rimraf(&target_bp)?;
            rimraf(&target_rp)?;
            copy_dir(bp, &target_bp)?;
            copy_dir(rp, &target_rp)?;
        }
        empty_dir(&temp)?;
        symlink(&target_bp, temp_bp)?;
        symlink(&target_rp, temp_rp)?;
        try_symlink(config.get_data_path(), temp_data)?;
    });

    measure_time!(profile_name, {
        info!("Running <b>{profile_name}</> profile");
        profile.run(&config, &temp, profile_name)?;
    });

    info!(
        "Applied changes to target location: \n\
         \tBP: {} \n\
         \tRP: {}",
        target_bp.display(),
        target_rp.display()
    );

    info!("Successfully ran the <b>{profile_name}</> profile");
    if watch {
        info!("Watching for changes...");
        info!("Press Ctrl+C to stop watching");
        config.watch_project_files()?;
        warn!("Changes detected, restarting...");
    }
    session.unlock()
}
