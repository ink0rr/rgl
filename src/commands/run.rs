use crate::fs::{copy_dir, empty_dir, move_dir, rimraf, try_symlink};
use crate::rgl::{copy_dir_cached, Config, Session};
use crate::{debug, info, measure_time, warn};
use anyhow::{Context, Result};
use std::time;

pub fn run_or_watch(profile_name: &str, watch: bool, cached: bool) -> Result<()> {
    let start_time = time::Instant::now();
    let config = Config::load()?;
    let mut session = Session::lock()?;

    let bp = config.get_behavior_pack();
    let rp = config.get_resource_pack();

    let profile = config.get_profile(profile_name)?;
    let (target_bp, target_rp, temp) = profile.get_export_paths(config.get_name())?;
    let temp_bp = temp.join("BP");
    let temp_rp = temp.join("RP");

    measure_time!("Setup temp dir", {
        empty_dir(&temp)?;
        if cached {
            copy_dir_cached(bp, &temp_bp, &target_bp)?;
            copy_dir_cached(rp, &temp_rp, &target_rp)?;
        } else {
            rimraf(&target_bp)?;
            rimraf(&target_rp)?;
            copy_dir(bp, &temp_bp)?;
            copy_dir(rp, &temp_rp)?;
        }
        try_symlink(config.get_data_path(), temp.join("data"))?;
    });

    measure_time!(profile_name, {
        info!("Running <b>{profile_name}</> profile");
        profile.run(&config, &temp, profile_name)?;
    });

    measure_time!("Export project", {
        info!(
            "Moving files to target location: \n\
             \tBP: {} \n\
             \tRP: {}",
            target_bp.display(),
            target_rp.display()
        );
        let export = || -> Result<()> {
            move_dir(temp_bp, target_bp)?;
            move_dir(temp_rp, target_rp)
        };
        export().context("Failed to export project")?;
    });

    info!("Successfully ran the <b>{profile_name}</> profile");
    debug!("Total time: {}ms", start_time.elapsed().as_millis());
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
