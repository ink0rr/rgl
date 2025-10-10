use super::UserConfig;
use crate::warn;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MinecraftBuild {
    Standard,
    Preview,
    Education,
}

fn mojang_dir() -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let home = PathBuf::from(env::var("HOME")?);
        let flatpak =
            home.join(".var/app/io.mrarm.mcpelauncher/data/mcpelauncher/games/com.mojang");
        if flatpak.exists() {
            return Ok(flatpak);
        }
        Ok(home.join(".local/share/mcpelauncher/games/com.mojang"))
    }
    #[cfg(target_os = "macos")]
    {
        let home = env::var("HOME")?;
        Ok(PathBuf::from(home).join("Library/Application Support/mcpelauncher/games/com.mojang"))
    }
    #[cfg(target_os = "windows")]
    {
        let appdata = env::var("AppData")?;
        let gdk =
            PathBuf::from(appdata).join("Minecraft Bedrock\\Users\\Shared\\games\\com.mojang");
        if gdk.exists() {
            return Ok(gdk);
        }
        let localappdata = env::var("LocalAppData")?;
        Ok(PathBuf::from(localappdata)
            .join("Packages\\Microsoft.MinecraftUWP_8wekyb3d8bbwe\\LocalState\\games\\com.mojang"))
    }
}

fn find_standard_mojang_dir() -> Result<PathBuf> {
    if let Ok(com_mojang) = env::var("COM_MOJANG") {
        return Ok(PathBuf::from(com_mojang));
    }
    if let Some(com_mojang) = UserConfig::mojang_dir() {
        warn!("User config `mojang_dir` is deprecated, use the `COM_MOJANG` environment variable instead");
        return Ok(PathBuf::from(com_mojang));
    }
    mojang_dir()
}

fn find_preview_mojang_dir() -> Result<PathBuf> {
    if let Ok(com_mojang) = env::var("COM_MOJANG_PREVIEW") {
        return Ok(PathBuf::from(com_mojang));
    }
    #[cfg(unix)]
    {
        mojang_dir()
    }
    #[cfg(windows)]
    {
        let appdata = env::var("AppData")?;
        let gdk = PathBuf::from(appdata)
            .join("Minecraft Bedrock Preview\\Users\\Shared\\games\\com.mojang");
        if gdk.exists() {
            return Ok(gdk);
        }
        let localappdata = env::var("LocalAppData")?;
        Ok(PathBuf::from(localappdata).join(
            "Packages\\Microsoft.MinecraftWindowsBeta_8wekyb3d8bbwe\\LocalState\\games\\com.mojang",
        ))
    }
}

fn find_education_mojang_dir() -> Result<PathBuf> {
    if let Ok(com_mojang) = env::var("COM_MOJANG_EDU") {
        return Ok(PathBuf::from(com_mojang));
    }
    #[cfg(unix)]
    {
        mojang_dir()
    }
    #[cfg(windows)]
    {
        let appdata = env::var("AppData")?;
        Ok(PathBuf::from(appdata).join("Minecraft Education Edition\\games\\com.mojang"))
    }
}

pub fn find_mojang_dir(build: Option<&MinecraftBuild>) -> Result<PathBuf> {
    match build {
        Some(MinecraftBuild::Standard) | None => find_standard_mojang_dir(),
        Some(MinecraftBuild::Preview) => find_preview_mojang_dir(),
        Some(MinecraftBuild::Education) => find_education_mojang_dir(),
    }
}

pub fn find_world_dir(build: Option<&MinecraftBuild>, world_name: &str) -> Result<PathBuf> {
    let mojang_dir = find_mojang_dir(build)?;
    if !mojang_dir.exists() {
        bail!("Failed to find com.mojang directory")
    }

    let mut worlds = HashMap::<String, PathBuf>::new();
    for entry in mojang_dir.join("minecraftWorlds").read_dir()? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path();
        let name = std::fs::read_to_string(path.join("levelname.txt"))?;
        if worlds.contains_key(&name) && name == world_name {
            bail!("Found more than one world named <yellow>{name}</>");
        }
        worlds.insert(name, path);
    }

    worlds
        .get(world_name)
        .cloned()
        .with_context(|| format!("World <yellow>{world_name}</> not found"))
}
