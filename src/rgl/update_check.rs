use crate::debug;
use crate::rgl::get_cache_dir;
use anyhow::{Context, Result};
use clap::crate_version;
use paris::log;
use std::{env, fs, path::PathBuf, time::SystemTime};

const CARGO_URL: &str = "https://raw.githubusercontent.com/ink0rr/rgl/master/Cargo.toml";

fn get_timestamp_path() -> Result<PathBuf> {
    let cache_dir = get_cache_dir()?;
    let path = cache_dir.join("latest.txt");
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

pub fn fetch_latest_version() -> Result<String> {
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

/// Check if there is a new version available.
pub fn check_for_update() -> Result<Option<String>> {
    let timestamp_path = get_timestamp_path()?;
    let last_update_check = timestamp_path.metadata()?.modified()?;
    let now = SystemTime::now();
    let elapsed_hour = now.duration_since(last_update_check)?.as_secs() / 60 / 60;

    debug!("Last update check: {elapsed_hour} hour(s) ago");
    if elapsed_hour > 24 {
        debug!("Fetching latest version info");
        let version = fetch_latest_version()?;
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
        log!("<bright-black><i>Run `rgl update` to install it");
    }
    update_timestamp()
}
