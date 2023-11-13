use super::{Config, FileWatcher, FilterDefinition, Profile};
use anyhow::{Context, Result};
use indexmap::IndexMap;
use std::collections::BTreeMap;

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

    pub fn watch_project_files(&self) -> Result<()> {
        let mut file_watcher = FileWatcher::new()?;

        file_watcher.watch(&self.data_path)?;
        file_watcher.watch(&self.behavior_pack)?;
        file_watcher.watch(&self.resource_pack)?;

        file_watcher.wait_changes();

        Ok(())
    }
}
