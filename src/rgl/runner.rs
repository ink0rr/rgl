use super::{Config, ExportPaths, Session, TcpChannel, TcpServerType, TcpTrait};
use crate::fs::{copy_dir, empty_dir, rimraf, symlink, sync_dir, try_symlink};
use crate::{error, info, measure_time, warn};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::{Mutex, Notify};

fn spawn_server(mut server_type: TcpServerType) -> Result<Arc<Notify>> {
    let server_status = Arc::new(Mutex::new(false));
    let channel = Arc::new(Mutex::new(TcpChannel::new()));
    let notifier = Arc::new(Notify::new());
    // Clone
    let server_status_c = Arc::clone(&server_status);
    let channel_c = Arc::clone(&channel);
    let notifier_c = Arc::clone(&notifier);
    // Spawn server
    spawn(async move {
        if let Err(e) = server_type.serve().await {
            error!("Could not start server: {}", e);
            return;
        }
        if let Err(e) = server_type
            .wait_for_client(Arc::clone(&channel_c), server_status_c)
            .await
        {
            error!("Server error: {}", e);
        }
    });
    // Spawn watcher
    spawn(async move {
        loop {
            notifier_c.notified().await;
            let v = server_status.lock().await;
            if *v == false {
                continue;
            }
            let mut ch = channel.lock().await;
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
    Ok(notifier)
}

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
    let mut notifier: Option<Arc<Notify>> = None;
    if let Some(server) = server_type {
        notifier = Some(spawn_server(server)?);
    }

    loop {
        let config = Config::load()?;
        let mut session = Session::lock()?;
        runner(&config, profile_name, cached)?;
        if let Some(notifier) = &notifier {
            notifier.notify_one();
        }
        info!("Watching for changes...");
        info!("Press Ctrl+C to stop watching");
        config.watch_project_files()?;
        warn!("Changes detected, restarting...");
        session.unlock()?;
    }
}
