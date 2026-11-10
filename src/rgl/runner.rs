use super::{check_export_path_collision, EditedFiles, Export, ExportPaths, RunContext};
use crate::fs::{rimraf, symlink, sync_dir};
use crate::{debug, info, measure_time};
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use url::Url;

pub async fn runner(ctx: &RunContext<'_>, clean: bool, compat: bool) -> Result<()> {
    let start = Instant::now();
    let config = ctx.config;
    let temp = ctx.temp;
    let profile_name = ctx.profile;
    let bp = config.get_behavior_pack();
    let rp = config.get_resource_pack();
    let data = config.get_data_path();

    let profile = config.get_profile(profile_name)?;
    let targets = profile.export.targets();

    // Resolve all active export paths up front and reject overlapping targets.
    let mut resolved: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut seen_paths: Vec<(PathBuf, String)> = Vec::new();
    for (i, target) in profile.export.active() {
        let (target_bp, target_rp) = target.get_paths(ctx).with_context(|| {
            format!(
                "Failed to get export paths for export target {} ({})",
                i + 1,
                target
            )
        })?;
        let label = format!("export target {} ({})", i + 1, target);
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
        resolved.push((target_bp, target_rp));
    }
    let is_none_export = resolved.is_empty();
    // Symlinking only makes sense with a single destination.
    let use_symlink = !compat && resolved.len() == 1;

    // With no active export target, fall back to the "none" target's paths.
    let (primary_bp, primary_rp) = match resolved.first() {
        Some(pair) => pair.clone(),
        None => targets[0].get_paths(ctx)?,
    };

    // Refuse to touch `exact` export targets containing files rgl didn't create.
    let exact_targets: Vec<&(PathBuf, PathBuf)> = profile
        .export
        .active()
        .zip(&resolved)
        .filter(|((_, target), _)| matches!(target, Export::Exact(_)))
        .map(|(_, paths)| paths)
        .collect();
    let mut edited_files = EditedFiles::load();
    for (target_bp, target_rp) in &exact_targets {
        measure_time!("Check deletion safety", {
            edited_files
                    .check_deletion_safety(target_bp, target_rp)
                    .with_context(|| {
                        format!(
                            "Safety mechanism stopped rgl to protect unexpected files from your export targets.\n\
                             Did you edit the exported files manually?\n\
                             Please clear your export paths and try again.\n\
                             <yellow> >></> Behavior pack export path: {}\n\
                             <yellow> >></> Resource pack export path: {}",
                            target_bp.display(),
                            target_rp.display()
                        )
                    })?;
        });
    }

    measure_time!("Setup temp", {
        if clean {
            rimraf(&temp.root)?;
            for (target_bp, target_rp) in &resolved {
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
            if let Some(bp) = &bp {
                sync_dir(bp, &primary_bp)?;
                symlink(&primary_bp, &temp.bp)?;
            }
            if let Some(rp) = &rp {
                sync_dir(rp, &primary_rp)?;
                symlink(&primary_rp, &temp.rp)?;
            }
            sync_dir(&data, &temp.data)?;

            // The target was just reseeded with raw source; record it now so a
            // filter failure doesn't make the next run's safety check reject it.
            if !exact_targets.is_empty() {
                edited_files.update_from_paths(&primary_bp, &primary_rp)?;
                edited_files.dump().context(
                    "Failed to update the list of files edited by rgl. This may cause the next run to fail.",
                )?;
            }
        }
    });
    smol::future::yield_now().await;

    measure_time!(profile_name, {
        info!("Running <profile>{profile_name}</> profile");
        let export_data_names = profile.run(ctx).await?;
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
            for (i, (target_bp, target_rp)) in resolved.iter().enumerate() {
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
                    }
                }
                if rp.is_some() {
                    print_export_line(&rp_label, target_rp);
                    if !use_symlink {
                        sync_dir(&temp.rp, target_rp)?;
                    }
                }
            }
        }
    });

    if !exact_targets.is_empty() {
        measure_time!("Update edited files", {
            for (target_bp, target_rp) in &exact_targets {
                edited_files.update_from_paths(target_bp, target_rp)?;
            }
            edited_files
                .dump()
                .context("Failed to update the list of files edited by rgl. This may cause the next run to fail.")?;
        });
    }

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
