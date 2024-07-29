use super::{Export, FilterDefinition, FilterRunner, Profile, UserConfig};
use crate::fs::{read_json, write_json};
use crate::watcher::Watcher;
use anyhow::{anyhow, Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    name: String,
    packs: Packs,
    regolith: Regolith,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Packs {
    behavior_pack: String,
    resource_pack: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Regolith {
    data_path: String,
    filter_definitions: BTreeMap<String, Value>,
    profiles: IndexMap<String, Profile>,
}

impl Config {
    pub fn new(name: String) -> Self {
        let mut profiles = IndexMap::<String, Profile>::new();
        profiles.insert(
            "default".to_owned(),
            Profile {
                export: Export {
                    target: "development".to_owned(),
                },
                filters: vec![],
            },
        );
        profiles.insert(
            "build".to_owned(),
            Profile {
                export: Export {
                    target: "local".to_owned(),
                },
                filters: vec![FilterRunner::ProfileFilter {
                    profile_name: "default".to_owned(),
                }],
            },
        );
        Self {
            schema: Some("https://raw.githubusercontent.com/Bedrock-OSS/regolith-schemas/main/config/v1.1.json".to_owned()),
            author: Some(UserConfig::username()),
            name,
            packs: Packs{
                behavior_pack: "./packs/BP".to_owned(),
                resource_pack: "./packs/RP".to_owned(),
            },
            regolith: Regolith {
                data_path: "./data".to_owned(),
                filter_definitions: BTreeMap::<String, Value>::new(),
                profiles,
            },
        }
    }

    pub fn load() -> Result<Self> {
        read_json("./config.json")
    }

    pub fn save(&self) -> Result<()> {
        write_json("./config.json", self)
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_behavior_pack(&self) -> PathBuf {
        PathBuf::from(&self.packs.behavior_pack)
    }

    pub fn get_resource_pack(&self) -> PathBuf {
        PathBuf::from(&self.packs.resource_pack)
    }

    pub fn get_data_path(&self) -> PathBuf {
        PathBuf::from(&self.regolith.data_path)
    }

    pub fn get_profile(&self, profile_name: &str) -> Result<&Profile> {
        self.regolith
            .profiles
            .get(profile_name)
            .context(format!("Profile <b>{profile_name}</> not found"))
    }

    pub fn get_filter(&self, filter_name: &str) -> Result<FilterDefinition> {
        let value = self
            .regolith
            .filter_definitions
            .get(filter_name)
            .context(format!(
                "Filter <b>{filter_name}</> is not defined in filterDefinitions"
            ))?
            .to_owned();
        FilterDefinition::from_value(value).map_err(|e| {
            anyhow!(
                "Invalid filter definition for <b>{filter_name}</>\n\
                 <yellow> >></> {e}"
            )
        })
    }

    pub fn get_filters(&self) -> Result<BTreeMap<String, FilterDefinition>> {
        let mut filters = BTreeMap::<String, FilterDefinition>::new();
        for (name, value) in &self.regolith.filter_definitions {
            let filter = FilterDefinition::from_value(value.to_owned()).map_err(|e| {
                anyhow!(
                    "Invalid filter definition for <b>{name}</>\n\
                     <yellow> >></> {e}"
                )
            })?;
            filters.insert(name.to_owned(), filter);
        }
        Ok(filters)
    }

    pub fn add_filter(&mut self, name: &str, filter: &FilterDefinition) -> Result<()> {
        self.regolith
            .filter_definitions
            .insert(name.to_owned(), serde_json::to_value(filter)?);
        Ok(())
    }

    pub fn remove_filter(&mut self, name: &str) -> Option<Value> {
        self.regolith.filter_definitions.remove(name)
    }

    pub fn watch_project_files(&self) -> Result<()> {
        let mut watcher = Watcher::new()?;

        watcher.watch("./config.json")?;
        watcher.watch(self.get_behavior_pack())?;
        watcher.watch(self.get_resource_pack())?;
        watcher.watch(self.get_data_path())?;

        watcher.wait_changes();

        Ok(())
    }
}
