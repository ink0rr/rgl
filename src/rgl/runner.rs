use super::{Config, Export, ExportPaths, Temp};
use crate::fs::{rimraf, symlink, sync_dir};
use crate::{debug, info, measure_time};
use anyhow::{Context, Result};
use std::fs;

pub fn runner(config: &Config, profile_name: &str, clean: bool, compat: bool) -> Result<()> {
    let bp = config.get_behavior_pack();
    let rp = config.get_resource_pack();
    let data = config.get_data_path();

    let profile = config.get_profile(profile_name)?;
    let (target_bp, target_rp) = profile
        .export
        .get_paths(config.get_name(), profile_name)
        .context("Failed to get export paths")?;
    let is_none_export = matches!(profile.export, Export::None(_));

    let temp = Temp::from_dot_regolith();

    measure_time!("Setup temp", {
        if !data.exists() {
            fs::create_dir_all(&data)?;
        }
        if clean {
            rimraf(&temp.root)?;
            rimraf(&target_bp)?;
            rimraf(&target_rp)?;
        }
        if compat || is_none_export {
            if temp.bp.is_symlink() {
                rimraf(&temp.bp)?;
            }
            if temp.rp.is_symlink() {
                rimraf(&temp.rp)?;
            }
            if temp.data.is_symlink() {
                rimraf(&temp.data)?;
            }
            sync_dir(bp, &temp.bp)?;
            sync_dir(rp, &temp.rp)?;
            sync_dir(&data, &temp.data)?;
        } else {
            rimraf(&temp.bp)?;
            rimraf(&temp.rp)?;
            if temp.data.is_symlink() {
                rimraf(&temp.data)?;
            }
            sync_dir(bp, &target_bp)?;
            sync_dir(rp, &target_rp)?;
            sync_dir(&data, &temp.data)?;
            symlink(&target_bp, &temp.bp)?;
            symlink(&target_rp, &temp.rp)?;
        }
    });

    measure_time!(profile_name, {
        info!("Running <b>{profile_name}</> profile");
        let export_data_names = profile.run(config, &temp.root, profile_name)?;
        for name in export_data_names {
            let filter_data = temp.data.join(&name);
            if filter_data.is_dir() {
                debug!("Exporting data for filter <b>{name}</>");
                sync_dir(filter_data, data.join(name))?;
            }
        }
    });

    measure_time!("Export project", {
        if compat && !is_none_export {
            sync_dir(&temp.bp, &target_bp)?;
            sync_dir(&temp.rp, &target_rp)?;
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
    Ok(())
}
