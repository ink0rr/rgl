use super::{Result, RglError};
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
pub fn find_mojang_dir() -> PathBuf {
    Path::new(env!("LOCALAPPDATA"))
        .join("Packages")
        .join("Microsoft.MinecraftUWP_8wekyb3d8bbwe")
        .join("LocalState")
        .join("games")
        .join("com.mojang")
}

pub fn find_temp_dir(target: &str) -> Result<PathBuf> {
    match target {
        "development" => Ok(find_mojang_dir().join(".regolith")),
        "local" => Ok(Path::new(".").join(".regolith").join("tmp")),
        _ => Err(RglError::ExportTargetError(target.to_owned())),
    }
}
