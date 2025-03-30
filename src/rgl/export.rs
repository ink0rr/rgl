use super::{find_education_mojang_dir, find_mojang_dir, find_preview_mojang_dir};
use anyhow::{bail, Result};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "target")]
#[enum_dispatch]
pub enum Export {
    Development(DevelopmentExport),
    Local(LocalExport),
}

#[enum_dispatch(Export)]
pub trait ExportPaths {
    fn get_paths(&self, name: &str) -> Result<(PathBuf, PathBuf)>;
}

#[derive(Default, Serialize, Deserialize)]
pub struct DevelopmentExport {
    #[serde(skip_serializing_if = "Option::is_none")]
    build: Option<MinecraftBuild>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MinecraftBuild {
    Standard,
    Preview,
    Education,
}

impl ExportPaths for DevelopmentExport {
    fn get_paths(&self, name: &str) -> Result<(PathBuf, PathBuf)> {
        let mojang_dir = match &self.build {
            Some(MinecraftBuild::Standard) | None => find_mojang_dir()?,
            Some(MinecraftBuild::Preview) => find_preview_mojang_dir()?,
            Some(MinecraftBuild::Education) => find_education_mojang_dir()?,
        };
        if !mojang_dir.exists() {
            bail!("Failed to find com.mojang directory")
        }
        let bp = mojang_dir
            .join("development_behavior_packs")
            .join(format!("{name}_bp"));
        let rp = mojang_dir
            .join("development_resource_packs")
            .join(format!("{name}_rp"));
        Ok((bp, rp))
    }
}

#[derive(Serialize, Deserialize)]
pub struct LocalExport {}

impl ExportPaths for LocalExport {
    fn get_paths(&self, name: &str) -> Result<(PathBuf, PathBuf)> {
        let build = PathBuf::from("build");
        if !build.exists() {
            fs::create_dir(&build)?;
        }
        let bp = build.join(format!("{name}_bp"));
        let rp = build.join(format!("{name}_rp"));
        Ok((bp, rp))
    }
}
