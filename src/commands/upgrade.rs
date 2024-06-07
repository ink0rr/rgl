use super::Command;
use crate::rgl::fetch_latest_version;
use crate::{info, loading, success};
use anyhow::{Context, Result};
use clap::{crate_version, Args};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
};
use tempfile::tempdir;
use zip::ZipArchive;

const TARGET: &str = env!("TARGET");

const RELEASE_URL: &str = "https://github.com/ink0rr/rgl/releases";

/// Checks for new rgl version and installs it if available
#[derive(Args)]
pub struct Upgrade {
    #[arg(short, long)]
    force: bool,
}

impl Command for Upgrade {
    fn dispatch(&self) -> Result<()> {
        info!("Looking up latest version");
        let current_version = crate_version!();
        let latest_version = fetch_latest_version()?;

        if !self.force && current_version == latest_version {
            info!("Already up to date");
            return Ok(());
        }
        info!("Found latest version: {latest_version}");
        let url = format!(
            "{}/download/v{}/rgl-{}.zip",
            RELEASE_URL, latest_version, TARGET
        );
        let bytes = download_pkg(&url).context(format!("Failed downloading {url}"))?;

        info!("Upgrading rgl to {latest_version}");
        let temp = tempdir()?;
        let current_exe_path = env::current_exe()?;
        let output_exe_path = extract_pkg(bytes, temp.path())?;
        let permissions = current_exe_path.metadata()?.permissions();

        replace_exe(&output_exe_path, &current_exe_path)
            .context("Failed to replace current executable with the new one")?;
        fs::set_permissions(current_exe_path, permissions)?;

        info!("Upgrade successful");
        info!("Release notes: {RELEASE_URL}/tag/v{latest_version}");
        Ok(())
    }
    fn error_context(&self) -> String {
        "Error upgrading rgl".to_owned()
    }
}

fn download_pkg(url: &str) -> Result<Vec<u8>> {
    loading!("Downloading {url}");

    let mut reader = ureq::get(url).call()?.into_reader();
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    success!("Downloaded {url}");
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
