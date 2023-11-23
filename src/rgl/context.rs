use super::{Config, FileWatcher, FilterDefinition, Profile};
use anyhow::{anyhow, Context, Result};
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
    pub fn new(config: Config, profile: &str) -> Result<Self> {
        let mut filter_definitions = BTreeMap::<String, FilterDefinition>::new();
        for (name, value) in config.regolith.filter_definitions {
            let filter = FilterDefinition::from_value(value).map_err(|e| {
                anyhow!(
                    "Invalid filter definition for <b>{name}</>\n\
                     <yellow> >></> {e}"
                )
            })?;
            filter_definitions.insert(name, filter);
        }
        let context = Self {
            name: config.name,
            behavior_pack: config.packs.behavior_pack,
            resource_pack: config.packs.resource_pack,
            data_path: config.regolith.data_path,
            filter_definitions,
            profiles: config.regolith.profiles,
            root_profile: profile.to_string(),
        };
        Ok(context)
    }

    pub fn get_profile(&self, profile_name: &str) -> Result<&Profile> {
        self.profiles
            .get(profile_name)
            .context(format!("Profile <b>{profile_name}</> not found"))
    }

    pub fn get_filter(&self, filter_name: &str) -> Result<&FilterDefinition> {
        self.filter_definitions.get(filter_name).context(format!(
            "Filter <b>{filter_name}</> is not defined in filter_definitions"
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
