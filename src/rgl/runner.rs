use super::{Config, Export, ExportPaths, Temp};
use crate::fs::{rimraf, symlink, sync_dir};
use crate::{debug, info, measure_time};
use anyhow::{Context, Result};
use std::{fs, time::Instant};
use url::Url;

pub async fn runner(config: &Config, profile_name: &str, clean: bool, compat: bool) -> Result<()> {
    let start = Instant::now();
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
        if clean {
            rimraf(&temp.root)?;
            rimraf(&target_bp)?;
            rimraf(&target_rp)?;
        }
        fs::create_dir_all(&data)?;
        fs::create_dir_all(&temp.root)?;
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
            if let Some(bp) = &bp {
                sync_dir(bp, &temp.bp)?;
            }
            if let Some(rp) = &rp {
                sync_dir(rp, &temp.rp)?;
            }
            sync_dir(&data, &temp.data)?;
        } else {
            rimraf(&temp.bp)?;
            rimraf(&temp.rp)?;
            if temp.data.is_symlink() {
                rimraf(&temp.data)?;
            }
            if let Some(bp) = &bp {
                sync_dir(bp, &target_bp)?;
                symlink(&target_bp, &temp.bp)?;
            }
            if let Some(rp) = &rp {
                sync_dir(rp, &target_rp)?;
                symlink(&target_rp, &temp.rp)?;
            }
            sync_dir(&data, &temp.data)?;
        }
    });
    smol::future::yield_now().await;

    measure_time!(profile_name, {
        info!("Running <profile>{profile_name}</> profile");
        let export_data_names = profile.run(config, &temp.root, profile_name).await?;
        for name in export_data_names {
            let filter_data = temp.data.join(&name);
            if filter_data.is_dir() {
                debug!("Exporting data for filter <filter>{name}</>");
                sync_dir(filter_data, data.join(name))?;
            }
        }
    });

    measure_time!("Export project", {
        info!("Exporting project to target location:");
        let export = compat && !is_none_export;
        if bp.is_some() {
            if target_bp.is_absolute() {
                let uri = Url::from_file_path(&target_bp).unwrap();
                println!("\tBP: {}", uri.as_str());
            } else {
                println!("\tBP: {}", target_bp.display());
            }
            if export {
                sync_dir(&temp.bp, &target_bp)?;
            }
        }
        if rp.is_some() {
            if target_rp.is_absolute() {
                let uri = Url::from_file_path(&target_rp).unwrap();
                println!("\tRP: {}", uri.as_str());
            } else {
                println!("\tRP: {}", target_rp.display());
            }
            if export {
                sync_dir(&temp.rp, &target_rp)?;
            }
        }
    });

    info!("Successfully ran the <profile>{profile_name}</> profile");
    info!("<green>Finished</> in {}ms", start.elapsed().as_millis());
    Ok(())
}
