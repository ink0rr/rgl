use super::{Config, FileWatcher, FilterDefinition, Profile, RglError, RglResult};
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

    pub fn get_profile(&self, profile_name: &str) -> RglResult<&Profile> {
        match self.profiles.get(profile_name) {
            Some(profile) => Ok(profile),
            None => Err(RglError::ProfileNotFound {
                profile_name: profile_name.to_owned(),
            }),
        }
    }

    pub fn get_filter_def(&self, filter_name: &str) -> RglResult<&FilterDefinition> {
        match self.filter_definitions.get(filter_name) {
            Some(filter_def) => Ok(filter_def),
            None => Err(RglError::FilterNotDefined {
                filter_name: filter_name.to_owned(),
            }),
        }
    }

    pub fn watch_project_files(&self) -> RglResult<()> {
        let mut file_watcher = FileWatcher::new();

        file_watcher.watch(&self.data_path)?;
        file_watcher.watch(&self.behavior_pack)?;
        file_watcher.watch(&self.resource_pack)?;

        file_watcher.wait_changes();

        Ok(())
    }
}
