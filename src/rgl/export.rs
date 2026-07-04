use super::{
    find_mojang_dir, find_world_dir, get_current_dir, get_dot_regolith_dir, Eval, MinecraftBuild,
};
use anyhow::{anyhow, bail, Result};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Component, PathBuf},
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "target")]
#[enum_dispatch]
pub enum Export {
    Development(DevelopmentExport),
    Local(LocalExport),
    Exact(ExactExport),
    None(NoneExport),
    World(WorldExport),
}

#[enum_dispatch(Export)]
pub trait ExportPaths {
    fn get_paths(&self, project_name: &str, profile_name: &str) -> Result<(PathBuf, PathBuf)>;
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevelopmentExport {
    #[serde(skip_serializing_if = "Option::is_none")]
    build: Option<MinecraftBuild>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bp_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rp_name: Option<String>,
}

impl ExportPaths for DevelopmentExport {
    fn get_paths(&self, project_name: &str, profile_name: &str) -> Result<(PathBuf, PathBuf)> {
        let mojang_dir = find_mojang_dir(self.build.as_ref())?;
        if !mojang_dir.exists() {
            bail!("Failed to find com.mojang directory")
        }
        let eval = Eval::new(profile_name, &get_current_dir()?, None);
        let bp = {
            let dir = mojang_dir.join("development_behavior_packs");
            if let Some(bp_name) = &self.bp_name {
                dir.join(eval.string(bp_name)?)
            } else {
                dir.join(format!("{project_name}_bp"))
            }
        };
        let rp = {
            let dir = mojang_dir.join("development_resource_packs");
            if let Some(rp_name) = &self.rp_name {
                dir.join(eval.string(rp_name)?)
            } else {
                dir.join(format!("{project_name}_rp"))
            }
        };
        Ok((bp, rp))
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalExport {
    #[serde(skip_serializing_if = "Option::is_none")]
    bp_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rp_name: Option<String>,
}

impl ExportPaths for LocalExport {
    fn get_paths(&self, project_name: &str, profile_name: &str) -> Result<(PathBuf, PathBuf)> {
        let build = PathBuf::from("build");
        if !build.exists() {
            fs::create_dir(&build)?;
        }
        let eval = Eval::new(profile_name, &get_current_dir()?, None);
        let bp = if let Some(bp_name) = &self.bp_name {
            build.join(eval.string(bp_name)?)
        } else {
            build.join(format!("{project_name}_bp"))
        };
        let rp = if let Some(rp_name) = &self.rp_name {
            build.join(eval.string(rp_name)?)
        } else {
            build.join(format!("{project_name}_rp"))
        };
        Ok((bp, rp))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExactExport {
    bp_path: String,
    rp_path: String,
}

impl ExportPaths for ExactExport {
    fn get_paths(&self, _project_name: &str, _profile_name: &str) -> Result<(PathBuf, PathBuf)> {
        let bp = resolve_path(&self.bp_path)?;
        let rp = resolve_path(&self.rp_path)?;
        if bp == rp {
            bail!("Both `bpPath` and `rpPath` resolved to the same path")
        }
        Ok((bp, rp))
    }
}

fn resolve_path(path: &str) -> Result<PathBuf> {
    let mut res = PathBuf::new();
    for component in PathBuf::from(path).components() {
        match component {
            Component::Normal(os_str) => {
                let part = os_str.to_string_lossy();
                if part.starts_with('%') && part.ends_with('%') {
                    let name = &part[1..part.len() - 1];
                    let value = env::var(name)
                        .map_err(|_| anyhow!("Environment variable <b>{name}</> not found"))?;
                    res.push(value);
                } else {
                    res.push(os_str);
                }
            }
            _ => res.push(component),
        }
    }
    fs::create_dir_all(&res).map_err(|_| {
        anyhow!(
            "Cannot create directory when a file with the same name exists\n\
             <yellow> >></> Path: {}",
            res.display()
        )
    })?;
    if res.join("config.json").is_file() {
        bail!(
            "The specified path is a project directory\n\
             <yellow> >></> Path: {}",
            res.display()
        )
    }
    Ok(dunce::canonicalize(res)?)
}

#[derive(Serialize, Deserialize)]
pub struct NoneExport {}

impl ExportPaths for NoneExport {
    fn get_paths(&self, _project_name: &str, _profile_name: &str) -> Result<(PathBuf, PathBuf)> {
        // Set the export target to temp just to not mess up the log messages
        let temp = get_dot_regolith_dir()?.join("tmp");
        Ok((temp.join("BP"), temp.join("RP")))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldExport {
    #[serde(skip_serializing_if = "Option::is_none")]
    build: Option<MinecraftBuild>,
    #[serde(skip_serializing_if = "Option::is_none")]
    world_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    world_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bp_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rp_name: Option<String>,
}

impl ExportPaths for WorldExport {
    fn get_paths(&self, project_name: &str, profile_name: &str) -> Result<(PathBuf, PathBuf)> {
        let world_dir = match (&self.world_name, &self.world_path) {
            (Some(world_name), None) => find_world_dir(self.build.as_ref(), world_name)?,
            (None, Some(world_path)) => resolve_path(world_path)?,
            (Some(_), Some(_)) => bail!("Using both `worldName` and `worldPath` is not allowed"),
            (None, None) => bail!(
                "The `world` export target requires either a `worldName` or `worldPath property`"
            ),
        };
        let eval = Eval::new(profile_name, &get_current_dir()?, None);
        let bp = {
            let dir = world_dir.join("behavior_packs");
            if let Some(bp_name) = &self.bp_name {
                dir.join(eval.string(bp_name)?)
            } else {
                dir.join(format!("{project_name}_bp"))
            }
        };
        let rp = {
            let dir = world_dir.join("resource_packs");
            if let Some(rp_name) = &self.rp_name {
                dir.join(eval.string(rp_name)?)
            } else {
                dir.join(format!("{project_name}_rp"))
            }
        };
        Ok((bp, rp))
    }
}
