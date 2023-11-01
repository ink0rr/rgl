use super::{copy_dir, empty_dir, export_project, find_temp_dir, symlink, Config, RunContext};
use crate::{info, warn};
use anyhow::{Context, Result};
use std::{fs, io};

pub fn run_or_watch(profile_name: &str, watch: bool) -> Result<()> {
    let config = Config::load()?;

    let context = RunContext::new(config, profile_name);
    let profile = context.get_profile(profile_name)?;
    let temp = find_temp_dir(&profile.export.target)?;

    empty_dir(&temp)?;
    copy_dir(&context.behavior_pack, temp.join("BP"))?;
    copy_dir(&context.resource_pack, temp.join("RP"))?;
    if let Err(e) = symlink(&context.data_path, temp.join("data")) {
        match e.downcast_ref::<io::Error>().map(|e| e.kind()) {
            Some(io::ErrorKind::NotFound) => {
                warn!("Data folder does not exists, creating...");
                fs::create_dir_all(&context.data_path)?;
            }
            _ => return Err(e),
        }
    }

    info!("Running <b>{profile_name}</> profile");
    profile.run(&context, &temp)?;

    export_project(&context.name, &temp, &profile.export.target)
        .context("Failed to export project")?;

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
