use super::{find_mojang_dir, Config, FileWatcher, FilterDefinition, Profile};
use anyhow::{bail, Context, Result};
use indexmap::IndexMap;
use std::{collections::BTreeMap, fs, path::PathBuf};

pub struct RunContext {
    pub name: String,
    pub behavior_pack: String,
    pub resource_pack: String,
    pub data_path: String,
    pub filter_definitions: BTreeMap<String, FilterDefinition>,
    pub profiles: IndexMap<String, Profile>,
    pub root_profile: String,
}

impl RunContext {
    pub fn new(config: Config, profile: &str) -> Self {
        Self {
            name: config.name,
            behavior_pack: config.packs.behavior_pack,
            resource_pack: config.packs.resource_pack,
            data_path: config.regolith.data_path,
            filter_definitions: config.regolith.filter_definitions,
            profiles: config.regolith.profiles,
            root_profile: profile.to_string(),
        }
    }

    pub fn get_profile(&self, profile_name: &str) -> Result<&Profile> {
        self.profiles
            .get(profile_name)
            .context(format!("Profile <b>{profile_name}</> not found"))
    }

    pub fn get_filter_def(&self, filter_name: &str) -> Result<&FilterDefinition> {
        self.filter_definitions.get(filter_name).context(format!(
            "Filter <b>{filter_name}</> not defined in filter_definitions"
        ))
    }

    pub fn get_export_paths(&self, profile: &Profile) -> Result<(PathBuf, PathBuf)> {
        let target = profile.export.target.as_str();
        match target {
            "development" => {
                let mojang_dir = find_mojang_dir()?;
                let bp = mojang_dir
                    .join("development_behavior_packs")
                    .join(format!("{}_bp", self.name));
                let rp = mojang_dir
                    .join("development_resource_packs")
                    .join(format!("{}_rp", self.name));
                Ok((bp, rp))
            }
            "local" => {
                let build = PathBuf::from("build");
                if !build.exists() {
                    fs::create_dir(&build)?;
                }
                let bp = build.join("BP");
                let rp = build.join("RP");
                Ok((bp, rp))
            }
            _ => bail!("Export target <b>{target}</> is not valid"),
        }
    }

    pub fn get_temp_dir(&self, profile: &Profile) -> Result<PathBuf> {
        let target = profile.export.target.as_str();
        match target {
            "development" => Ok(find_mojang_dir()?.join(".regolith").join(&self.name)),
            _ => Ok(PathBuf::from(".regolith").join("tmp")),
        }
    }

    pub fn watch_project_files(&self) -> Result<()> {
        let mut file_watcher = FileWatcher::new();

        file_watcher.watch(&self.data_path)?;
        file_watcher.watch(&self.behavior_pack)?;
        file_watcher.watch(&self.resource_pack)?;

        file_watcher.wait_changes();

        Ok(())
    }
}
