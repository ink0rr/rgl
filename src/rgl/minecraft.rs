use super::UserConfig;
use anyhow::Result;
use std::env;
use std::path::PathBuf;

#[cfg(target_os = "linux")]
pub fn mojang_dir() -> Result<PathBuf> {
    let home = env::var("HOME")?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("mcpelauncher")
        .join("games")
        .join("com.mojang"))
}

#[cfg(target_os = "macos")]
pub fn mojang_dir() -> Result<PathBuf> {
    let home = env::var("HOME")?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("mcpelauncher")
        .join("games")
        .join("com.mojang"))
}

#[cfg(target_os = "windows")]
fn mojang_dir() -> Result<PathBuf> {
    let localappdata = env::var("LocalAppData")?;
    Ok(PathBuf::from(localappdata)
        .join("Packages")
        .join("Microsoft.MinecraftUWP_8wekyb3d8bbwe")
        .join("LocalState")
        .join("games")
        .join("com.mojang"))
}

pub fn find_mojang_dir() -> Result<PathBuf> {
    if let Some(com_mojang) = UserConfig::mojang_dir() {
        return Ok(PathBuf::from(com_mojang));
    }
    mojang_dir()
}

#[cfg(unix)]
pub fn find_preview_mojang_dir() -> Result<PathBuf> {
    mojang_dir()
}

#[cfg(target_os = "windows")]
fn find_preview_mojang_dir() -> Result<PathBuf> {
    let localappdata = env::var("LocalAppData")?;
    Ok(PathBuf::from(localappdata)
        .join("Packages")
        .join("Microsoft.MinecraftWindowsBeta_8wekyb3d8bbwe")
        .join("LocalState")
        .join("games")
        .join("com.mojang"))
}

#[cfg(unix)]
pub fn find_education_mojang_dir() -> Result<PathBuf> {
    mojang_dir()
}

#[cfg(target_os = "windows")]
fn find_education_mojang_dir() -> Result<PathBuf> {
    let appdata = env::var("AppData")?;
    Ok(PathBuf::from(appdata)
        .join("Minecraft Education Edition")
        .join("games")
        .join("com.mojang"))
}
