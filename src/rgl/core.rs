use super::{copy_dir, copy_dir_cached, empty_dir, move_dir, rimraf, symlink, Config, RunContext};
use crate::{debug, info, measure_time, rgl::Session, warn};
use anyhow::{Context, Result};
use std::{fs, io, time};

pub fn run_or_watch(profile_name: &str, watch: bool, cached: bool) -> Result<()> {
    let start_time = time::Instant::now();
    let config = Config::load()?;

    let mut session = Session::lock()?;
    let context = RunContext::new(config, profile_name)?;
    let profile = context.get_profile(profile_name)?;
    let (bp, rp, temp) = profile.get_export_paths(&context.name)?;
    let temp_bp = temp.join("BP");
    let temp_rp = temp.join("RP");

    measure_time!("Setup temp dir", {
        empty_dir(&temp)?;
        if cached {
            copy_dir_cached(&context.behavior_pack, &temp_bp, &bp)?;
            copy_dir_cached(&context.resource_pack, &temp_rp, &rp)?;
        } else {
            rimraf(&bp)?;
            rimraf(&rp)?;
            copy_dir(&context.behavior_pack, &temp_bp)?;
            copy_dir(&context.resource_pack, &temp_rp)?;
        }
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
        let export = || -> Result<()> {
            move_dir(temp_bp, bp)?;
            move_dir(temp_rp, rp)
        };
        export().context("Failed to export project")?;
    });

    info!("Successfully ran the <b>{profile_name}</> profile");
    debug!("Total time: {}ms", start_time.elapsed().as_millis());
    if watch {
        info!("Watching for changes...");
        info!("Press Ctrl+C to stop watching");
        context.watch_project_files()?;
        warn!("Changes detected, restarting...");
        session.unlock()?;
        return run_or_watch(profile_name, watch, cached);
    }
    session.unlock()
}
