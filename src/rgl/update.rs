use super::get_regolith_cache_dir;
use crate::{debug, info};
use anyhow::{Context, Result};
use clap::crate_version;
use paris::log;
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    time::SystemTime,
};
use tempfile::tempdir;
use zip::ZipArchive;

const TARGET: &str = env!("TARGET");

// How often it should query server for new version. In hours.
const UPDATE_CHECK_INTERVAL: u64 = 24;

const CARGO_URL: &str = "https://raw.githubusercontent.com/ink0rr/rgl/master/Cargo.toml";

const RELEASE_URL: &str = "https://github.com/ink0rr/rgl/releases";

fn get_timestamp_path() -> Result<PathBuf> {
    let cache_dir = get_regolith_cache_dir()?;
    let path = cache_dir.join("rgl_latest.txt");
    if !path.exists() {
        fs::create_dir_all(cache_dir)?;
        fs::write(&path, b"")?;
    }
    Ok(path)
}

fn update_timestamp() -> Result<()> {
    let path = get_timestamp_path()?;
    fs::write(path, b"")?;
    Ok(())
}

fn get_latest_version() -> Result<String> {
    let version = ureq::get(CARGO_URL)
        .call()
        .context("Failed to fetch Cargo.toml")?
        .into_string()
        .context("Failed to parse Cargo.toml")?
        .lines()
        .find(|line| line.starts_with("version = \""))
        .context("Failed to find version in Cargo.toml")?
        .trim_start_matches("version = ")
        .trim_matches('"')
        .to_string();
    Ok(version)
}

fn download_pkg(url: &str) -> Result<Vec<u8>> {
    let mut logger = paris::Logger::new();
    logger.loading(format!("Downloading {url}"));

    let mut reader = ureq::get(url).call()?.into_reader();
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    logger.log(format!("<green>[SUCCESS]</> Downloaded {url}"));
    Ok(bytes)
}

fn extract_pkg(bytes: Vec<u8>, path: &Path) -> Result<PathBuf> {
    let mut archive = ZipArchive::new(io::Cursor::new(bytes))?;
    let mut file = archive.by_index(0)?;

    let mangled_name = file.mangled_name();
    let file_name = mangled_name.file_name().unwrap();

    let file_path = path.join(file_name);
    let mut outfile = fs::File::create(&file_path)?;
    io::copy(&mut file, &mut outfile)?;

    Ok(file_path)
}

// Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.
fn replace_exe(from: &Path, to: &Path) -> Result<(), io::Error> {
    if cfg!(windows) {
        // On windows you cannot replace the currently running executable.
        // so first we rename it to rgl.old.exe
        fs::rename(to, to.with_extension("old.exe"))?;
    } else {
        fs::remove_file(to)?;
    }
    // Windows cannot rename files across device boundaries, so if rename fails,
    // we try again with copy.
    fs::rename(from, to).or_else(|_| fs::copy(from, to).map(|_| ()))?;
    Ok(())
}

/// Check if there is a new version available.
pub fn check_for_update() -> Result<Option<String>> {
    let timestamp_path = get_timestamp_path()?;
    let last_update_check = timestamp_path.metadata()?.modified()?;
    let now = SystemTime::now();
    let elapsed_hour = now.duration_since(last_update_check)?.as_secs() / 60 / 60;

    debug!("Last update check: {elapsed_hour} hour(s) ago");
    if elapsed_hour > UPDATE_CHECK_INTERVAL {
        debug!("Fetching latest version info");
        let version = get_latest_version()?;
        Ok(Some(version))
    } else {
        Ok(None)
    }
}

/// Prompt the user to update if there is a new version available.
pub fn prompt_update(latest_version: String) -> Result<()> {
    let current_version = crate_version!();
    if current_version != latest_version {
        log!("<green>A new version of rgl is available: <cyan>{current_version}</> → <cyan>{latest_version}");
        log!("<black><i>Run `rgl update` to install it");
    }
    update_timestamp()
}

/// Update the current executable to the latest version.
pub fn update(force: bool) -> Result<()> {
    info!("Looking up latest version");
    let current_version = crate_version!();
    let latest_version = get_latest_version()?;

    if !force && current_version == latest_version {
        info!("Already up to date");
        return Ok(());
    }
    info!("Found latest version: {latest_version}");
    let url = format!(
        "{}/download/v{}/rgl-{}.zip",
        RELEASE_URL, latest_version, TARGET
    );
    let bytes = download_pkg(&url).context(format!("Failed downloading {url}"))?;

    info!("Updating rgl to {latest_version}");
    let temp = tempdir()?;
    let current_exe_path = env::current_exe()?;
    let output_exe_path = extract_pkg(bytes, temp.path())?;
    let permissions = current_exe_path.metadata()?.permissions();

    replace_exe(&output_exe_path, &current_exe_path)
        .context("Failed to replace current executable with the new one")?;
    fs::set_permissions(current_exe_path, permissions)?;

    info!("Update successful");
    update_timestamp()
}
