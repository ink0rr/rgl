use super::{check_export_path_collision, Config, ExportPaths, Temp};
use crate::fs::{rimraf, set_readonly_recursive, symlink, sync_dir};
use crate::{debug, info, measure_time};
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use url::Url;

pub async fn runner(
    config: &Config,
    profile_name: &str,
    clean: bool,
    compat: bool,
    extra_args: &[String],
) -> Result<()> {
    let start = Instant::now();
    let bp = config.get_behavior_pack();
    let rp = config.get_resource_pack();
    let data = config.get_data_path();

    let profile = config.get_profile(profile_name)?;
    let targets = profile.export.targets();

    // Resolve every active (non-"none") export target's paths before
    // touching the filesystem, and reject the run if any two targets would
    // collide (resolve to the same, or an overlapping, destination). This
    // mirrors Go's `checkExportPathCollision`.
    let mut resolved: Vec<(PathBuf, PathBuf, bool)> = Vec::new();
    let mut seen_paths: Vec<(PathBuf, String)> = Vec::new();
    for (i, target) in profile.export.active() {
        let (target_bp, target_rp) = target
            .get_paths(config.get_name(), profile_name)
            .with_context(|| {
                format!(
                    "Failed to get export paths for export target {} ({})",
                    i + 1,
                    target.kind()
                )
            })?;
        let label = format!("export target {} ({})", i + 1, target.kind());
        check_export_path_collision(
            &mut seen_paths,
            &target_bp,
            &format!("{label} behavior pack: {}", target_bp.display()),
        )?;
        check_export_path_collision(
            &mut seen_paths,
            &target_rp,
            &format!("{label} resource pack: {}", target_rp.display()),
        )?;
        resolved.push((target_bp, target_rp, target.read_only()));
    }
    let is_none_export = resolved.is_empty();
    // Symlinking straight into the export target only makes sense when
    // there's exactly one destination to point at.
    let use_symlink = !compat && resolved.len() == 1;

    // Used to set up `tmp/` below. When there's no active export target,
    // fall back to the first (necessarily "none") target's paths, same as
    // the historical single-export behavior.
    let (primary_bp, primary_rp) = match resolved.first() {
        Some((bp, rp, _)) => (bp.clone(), rp.clone()),
        None => targets[0].get_paths(config.get_name(), profile_name)?,
    };

    let temp = Temp::from_dot_regolith()?;

    measure_time!("Setup temp", {
        if clean {
            rimraf(&temp.root)?;
            for (target_bp, target_rp, _) in &resolved {
                rimraf(target_bp)?;
                rimraf(target_rp)?;
            }
        }
        fs::create_dir_all(&data)?;
        fs::create_dir_all(&temp.root)?;
        if !use_symlink {
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
                sync_dir(bp, &primary_bp)?;
                symlink(&primary_bp, &temp.bp)?;
            }
            if let Some(rp) = &rp {
                sync_dir(rp, &primary_rp)?;
                symlink(&primary_rp, &temp.rp)?;
            }
            sync_dir(&data, &temp.data)?;
        }
    });
    smol::future::yield_now().await;

    measure_time!(profile_name, {
        info!("Running <profile>{profile_name}</> profile");
        let export_data_names = profile
            .run(config, &temp.root, profile_name, extra_args)
            .await?;
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
        if is_none_export {
            if bp.is_some() {
                print_export_line("BP", &primary_bp);
            }
            if rp.is_some() {
                print_export_line("RP", &primary_rp);
            }
        } else {
            let multiple = resolved.len() > 1;
            // In symlink mode the export dir *is* the working directory the
            // filters just ran in (temp.bp/temp.rp symlink straight into
            // it), so `readOnly` is not applied there: doing so would leave
            // read-only files behind for the filters to write into on the
            // next run. This mirrors Go's `ExportProject`, which skips the
            // whole export step (`setPathReadOnly` included) for the
            // symlinked target.
            for (i, (target_bp, target_rp, read_only)) in resolved.iter().enumerate() {
                let bp_label = if multiple {
                    format!("BP (target {})", i + 1)
                } else {
                    "BP".to_owned()
                };
                let rp_label = if multiple {
                    format!("RP (target {})", i + 1)
                } else {
                    "RP".to_owned()
                };
                if bp.is_some() {
                    print_export_line(&bp_label, target_bp);
                    if !use_symlink {
                        sync_dir(&temp.bp, target_bp)?;
                        if *read_only {
                            set_readonly_recursive(target_bp)?;
                        }
                    }
                }
                if rp.is_some() {
                    print_export_line(&rp_label, target_rp);
                    if !use_symlink {
                        sync_dir(&temp.rp, target_rp)?;
                        if *read_only {
                            set_readonly_recursive(target_rp)?;
                        }
                    }
                }
            }
        }
    });

    info!("Successfully ran the <profile>{profile_name}</> profile");
    info!("<green>Finished</> in {}ms", start.elapsed().as_millis());
    Ok(())
}

fn print_export_line(label: &str, path: &Path) {
    if path.is_absolute() {
        let uri = Url::from_file_path(path).unwrap();
        println!("\t{label}: {}", uri.as_str());
    } else {
        println!("\t{label}: {}", path.display());
    }
}
