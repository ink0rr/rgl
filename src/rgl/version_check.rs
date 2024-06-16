use crate::rgl::get_timestamp_path;
use crate::{debug, log};
use anyhow::{Context, Result};
use clap::crate_version;
use std::{
    env, fs,
    path::PathBuf,
    time::{Duration, SystemTime},
};

const CARGO_URL: &str = "https://raw.githubusercontent.com/ink0rr/rgl/master/Cargo.toml";

fn get_timestamp() -> Result<PathBuf> {
    let path = get_timestamp_path()?;
    if !path.exists() {
        fs::write(&path, b"")?;
    }
    Ok(path)
}

fn update_timestamp() -> Result<()> {
    let path = get_timestamp()?;
    fs::write(path, b"")?;
    Ok(())
}

pub fn fetch_latest_version() -> Result<String> {
    let version = ureq::get(CARGO_URL)
        .timeout(Duration::from_secs(10))
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
pub fn version_check() -> Result<Option<String>> {
    let timestamp = get_timestamp()?;
    let last_checked = timestamp.metadata()?.modified()?;
    let now = SystemTime::now();
    let elapsed_hour = now.duration_since(last_checked)?.as_secs() / 60 / 60;

    debug!("Last version check: {elapsed_hour} hour(s) ago");
    if elapsed_hour > 24 {
        debug!("Fetching latest version info");
        let version = fetch_latest_version()?;
        Ok(Some(version))
    } else {
        Ok(None)
    }
}

/// Prompt the user to upgrade if there is a new version available.
pub fn prompt_upgrade(latest_version: String) -> Result<()> {
    let current_version = crate_version!();
    if current_version != latest_version {
        log!("<green>A new version of rgl is available: <cyan>{current_version}</> → <cyan>{latest_version}</>");
        log!("<bright-black><i>Run `rgl upgrade` to install it</>");
    }
    update_timestamp()
}
