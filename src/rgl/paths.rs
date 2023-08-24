use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
pub fn find_mojang_dir() -> PathBuf {
    Path::new(env!("HOME"))
        .join(".local")
        .join("share")
        .join("mcpelauncher")
        .join("games")
        .join("com.mojang")
}

#[cfg(target_os = "macos")]
pub fn find_mojang_dir() -> PathBuf {
    Path::new(env!("HOME"))
        .join("Library")
        .join("Application Support")
        .join("mcpelauncher")
        .join("games")
        .join("com.mojang")
}

#[cfg(target_os = "windows")]
pub fn find_mojang_dir() -> PathBuf {
    Path::new(env!("LOCALAPPDATA"))
        .join("Packages")
        .join("Microsoft.MinecraftUWP_8wekyb3d8bbwe")
        .join("LocalState")
        .join("games")
        .join("com.mojang")
}

pub fn find_temp_dir(target: &str) -> PathBuf {
    match target {
        "development" => find_mojang_dir().join(".regolith"),
        _ => Path::new(".").join(".regolith").join("tmp"),
    }
}
