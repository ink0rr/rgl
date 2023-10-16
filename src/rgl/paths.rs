use anyhow::Result;
use std::env;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
pub fn find_mojang_dir() -> Result<PathBuf> {
    let home = env::var("HOME")?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("mcpelauncher")
        .join("games")
        .join("com.mojang"))
}

#[cfg(target_os = "macos")]
pub fn find_mojang_dir() -> Result<PathBuf> {
    let home = env::var("HOME")?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("mcpelauncher")
        .join("games")
        .join("com.mojang"))
}

#[cfg(target_os = "windows")]
pub fn find_mojang_dir() -> Result<PathBuf> {
    let localappdata = env::var("LocalAppData")?;
    Ok(PathBuf::from(localappdata)
        .join("Packages")
        .join("Microsoft.MinecraftUWP_8wekyb3d8bbwe")
        .join("LocalState")
        .join("games")
        .join("com.mojang"))
}

pub fn find_temp_dir(target: &str) -> Result<PathBuf> {
    match target {
        "development" => Ok(find_mojang_dir()?.join(".regolith")),
        _ => Ok(Path::new(".").join(".regolith").join("tmp")),
    }
}
