use super::{Config, ExportPaths, Session};
use crate::fs::{rimraf, symlink, sync_dir};
use crate::{debug, info, measure_time, warn};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub fn run_or_watch(profile_name: &str, watch: bool, clean: bool, compat: bool) -> Result<()> {
    let config = Config::load()?;
    let mut session = Session::lock()?;

    let bp = config.get_behavior_pack();
    let rp = config.get_resource_pack();
    let data = config.get_data_path();
    let dot_regolith = PathBuf::from(".regolith");

    let profile = config.get_profile(profile_name)?;
    let (target_bp, target_rp) = profile
        .export
        .get_paths(config.get_name(), profile_name)
        .context("Failed to get export paths")?;

    let temp = dot_regolith.join("tmp");
    let temp_bp = temp.join("BP");
    let temp_rp = temp.join("RP");
    let temp_data = temp.join("data");

    measure_time!("Setup temp", {
        if !data.exists() {
            fs::create_dir_all(&data)?;
        }
        if clean {
            rimraf(&temp)?;
            rimraf(&target_bp)?;
            rimraf(&target_rp)?;
        }
        if compat {
            if temp_bp.is_symlink() {
                rimraf(&temp_bp)?;
            }
            if temp_rp.is_symlink() {
                rimraf(&temp_rp)?;
            }
            if temp_data.is_symlink() {
                rimraf(&temp_data)?;
            }
            sync_dir(bp, &temp_bp)?;
            sync_dir(rp, &temp_rp)?;
            sync_dir(&data, &temp_data)?;
        } else {
            rimraf(&temp_bp)?;
            rimraf(&temp_rp)?;
            if temp_data.is_symlink() {
                rimraf(&temp_data)?;
            }
            sync_dir(bp, &target_bp)?;
            sync_dir(rp, &target_rp)?;
            sync_dir(&data, &temp_data)?;
            symlink(&target_bp, &temp_bp)?;
            symlink(&target_rp, &temp_rp)?;
        }
    });

    measure_time!(profile_name, {
        info!("Running <b>{profile_name}</> profile");
        let export_data_names = profile.run(&config, &temp, profile_name)?;
        for name in export_data_names {
            let filter_data = temp_data.join(&name);
            if filter_data.is_dir() {
                debug!("Exporting data for filter <b>{name}</>");
                sync_dir(filter_data, data.join(name))?;
            }
        }
    });

    measure_time!("Export project", {
        if compat {
            sync_dir(&temp_bp, &target_bp)?;
            sync_dir(&temp_rp, &target_rp)?;
        }
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
