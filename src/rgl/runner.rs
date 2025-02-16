use super::{Config, ExportPaths, Session, TcpChannel, TcpServerType, TcpTrait};
use crate::fs::{copy_dir, empty_dir, rimraf, symlink, sync_dir, try_symlink};
use crate::{error, info, measure_time, warn};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::{Mutex, Notify};

fn runner(config: &Config, profile_name: &str, cached: bool) -> Result<()> {
    let bp = config.get_behavior_pack();
    let rp = config.get_resource_pack();

    let profile = config.get_profile(profile_name)?;
    let (target_bp, target_rp) = profile.export.get_paths(config.get_name())?;

    let temp = PathBuf::from(".regolith").join("tmp");
    let temp_bp = temp.join("BP");
    let temp_rp = temp.join("RP");
    let temp_data = temp.join("data");

    measure_time!("Setup temp", {
        if cached {
            sync_dir(bp, &target_bp)?;
            sync_dir(rp, &target_rp)?;
        } else {
            rimraf(&target_bp)?;
            rimraf(&target_rp)?;
            copy_dir(bp, &target_bp)?;
            copy_dir(rp, &target_rp)?;
        }
        empty_dir(&temp)?;
        symlink(&target_bp, temp_bp)?;
        symlink(&target_rp, temp_rp)?;
        try_symlink(config.get_data_path(), temp_data)?;
    });

    measure_time!(profile_name, {
        info!("Running <b>{profile_name}</> profile");
        profile.run(&config, &temp, profile_name)?;
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

pub fn run(profile_name: &str, cached: bool) -> Result<()> {
    let config = Config::load()?;
    let mut session = Session::lock()?;
    runner(&config, profile_name, cached)?;
    session.unlock()
}

pub fn watch(profile_name: &str, cached: bool, server_type: Option<TcpServerType>) -> Result<()> {
    let notify = Arc::new(Notify::new());
    let mut has_server = false;
    if let Some(mut server) = server_type {
        has_server = true;
        let ch = Arc::new(Mutex::new(TcpChannel::new()));
        let notify_cloned = Arc::clone(&notify);
        let ch_cloned = Arc::clone(&ch);
        spawn(async move {
            if let Err(e) = server.serve().await {
                error!("Could not start server: {}", e);
                return;
            }
            if let Err(e) = server.wait_for_client(Arc::clone(&ch_cloned)).await {
                error!("Server error: {}", e);
            }
        });
        spawn(async move {
            loop {
                notify_cloned.notified().await;
                let mut ch = ch.lock().await;
                warn!("Auto reloading...");
                let commands = vec![
                    "playsound note.bell @a",
                    "tellraw @a {\\\"rawtext\\\":[{\\\"text\\\":\\\"§6[rgl] §aAuto Reload...§r\\\"}]}",
                    "reload",
                ];
                for command in commands {
                    ch.send_command(command).await.unwrap();
                }
            }
        });
    }

    loop {
        let config = Config::load()?;
        let mut session = Session::lock()?;
        runner(&config, profile_name, cached)?;

        if has_server {
            let notify = Arc::clone(&notify);
            notify.notify_one();
        }

        info!("Watching for changes...");
        info!("Press Ctrl+C to stop watching");
        config.watch_project_files()?;
        warn!("Changes detected, restarting...");
        session.unlock()?;
    }
}
