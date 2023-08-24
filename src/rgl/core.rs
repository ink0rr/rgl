use super::{
    copy_dir, empty_dir, export_project, find_temp_dir, get_config, symlink, RglError, RglResult,
    RunContext,
};
use simplelog::{info, warn};

pub fn run_or_watch(profile_name: &str, watch: bool) -> RglResult<()> {
    let config = get_config()?;

    let context = RunContext::new(config, profile_name);
    let profile = context.get_profile(profile_name)?;
    let temp = find_temp_dir(&profile.export.target);

    empty_dir(&temp)?;
    copy_dir(&context.behavior_pack, temp.join("BP"))?;
    copy_dir(&context.resource_pack, temp.join("RP"))?;
    symlink(&context.data_path, temp.join("data"))?;

    info!("Running <b>{profile_name}</> profile");
    profile.run(&context, &temp)?;

    if let Err(e) = export_project(&context.name, &temp, &profile.export.target) {
        return Err(RglError::ExportError { cause: e.into() });
    }

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
