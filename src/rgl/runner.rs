use crate::fs::{copy_dir, empty_dir, move_dir, rimraf, sync_dir, try_symlink};
use crate::rgl::{Config, Session};
use crate::{info, measure_time, warn};
use anyhow::{Context, Result};
use std::fs::create_dir_all;

pub fn run_or_watch(profile_name: &str, watch: bool, cached: bool) -> Result<()> {
    let config = Config::load()?;
    let mut session = Session::lock()?;

    let bp = config.get_behavior_pack();
    let rp = config.get_resource_pack();

    let profile = config.get_profile(profile_name)?;
    let (target_bp, target_rp, temp) = profile.get_export_paths(config.get_name())?;
    let temp_bp = temp.join("BP");
    let temp_rp = temp.join("RP");

    measure_time!("Setup temp", {
        if cached {
            create_dir_all(&temp)?;
            if !temp_bp.is_symlink() {
                if temp_bp.exists() {
                    rimraf(&temp_bp)?;
                }
                try_symlink(&target_bp, &temp_bp)?;
            }
            if !temp_rp.is_symlink() {
                if temp_rp.exists() {
                    rimraf(&temp_rp)?;
                }
                try_symlink(&target_rp, &temp_rp)?;
            }
            sync_dir(bp, &target_bp)?;
            sync_dir(rp, &target_rp)?;
        } else {
            empty_dir(&temp)?;
            rimraf(&target_bp)?;
            rimraf(&target_rp)?;
            copy_dir(bp, &temp_bp)?;
            copy_dir(rp, &temp_rp)?;
        }
        let data_path = temp.join("data");
        if !data_path.is_symlink() {
            try_symlink(config.get_data_path(), data_path)?;
        }
    });

    measure_time!(profile_name, {
        info!("Running <b>{profile_name}</> profile");
        profile.run(&config, &temp, profile_name)?;
    });

    info!(
        "Moving files to target location: \n\
         \tBP: {} \n\
         \tRP: {}",
        target_bp.display(),
        target_rp.display()
    );
    let export = || -> Result<()> {
        if !cached {
            move_dir(temp_bp, target_bp)?;
            move_dir(temp_rp, target_rp)?;
        }
        Ok(())
    };
    export().context("Failed to export project")?;

    info!("Successfully ran the <b>{profile_name}</> profile");
    if watch {
        info!("Watching for changes...");
        info!("Press Ctrl+C to stop watching");
        config.watch_project_files()?;
        warn!("Changes detected, restarting...");
        session.unlock()?;
        return run_or_watch(profile_name, watch, cached);
    }
    session.unlock()
}
