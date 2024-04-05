use anyhow::Result;
use std::env;
use std::path::PathBuf;

#[cfg(target_os = "linux")]
pub fn find_mojang_dir() -> Result<PathBuf> {
    if let Ok(com_mojang) = env::var("COM_MOJANG") {
        return Ok(PathBuf::from(com_mojang));
    }
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
    if let Ok(com_mojang) = env::var("COM_MOJANG") {
        return Ok(PathBuf::from(com_mojang));
    }
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
