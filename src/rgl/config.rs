use super::{
    DevelopmentExport, Export, FilterDefinition, FilterRunner, LocalExport, Profile, ProfileEntry,
    RemoteFilter, UserConfig,
};
use crate::file_watcher::FileWatcher;
use crate::fs::{read_json, write_file, write_json};
use anyhow::{anyhow, bail, Context, Result};
use indexmap::IndexMap;
use jsonc_parser::cst::{CstObject, CstRootNode};
use jsonc_parser::{json, ParseOptions};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    behavior_pack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_pack: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Regolith {
    data_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    watch_paths: Option<Vec<String>>,
    filter_definitions: BTreeMap<String, Value>,
    profiles: IndexMap<String, Profile>,
}

impl Config {
    pub fn new(name: String) -> Self {
        let mut profiles = IndexMap::<String, Profile>::new();
        profiles.insert(
            "default".to_owned(),
            Profile {
                export: Export::Development(DevelopmentExport::default()),
                filters: vec![],
            },
        );
        profiles.insert(
            "build".to_owned(),
            Profile {
                export: Export::Local(LocalExport::default()),
                filters: vec![ProfileEntry::Filter(FilterRunner::ProfileFilter {
                    profile_name: "default".to_owned(),
                })],
            },
        );
        Self {
            schema: Some(
                "https://raw.githubusercontent.com/ink0rr/rgl-schemas/main/config/v1.json"
                    .to_owned(),
            ),
            author: Some(UserConfig::username()),
            name,
            packs: Packs {
                behavior_pack: Some("./packs/BP".to_owned()),
                resource_pack: Some("./packs/RP".to_owned()),
            },
            regolith: Regolith {
                data_path: "./data".to_owned(),
                watch_paths: None,
                filter_definitions: BTreeMap::<String, Value>::new(),
                profiles,
            },
        }
    }

    pub fn load() -> Result<Self> {
        let config = read_json::<Config>("./config.json")?;
        if config.packs.behavior_pack.is_none() && config.packs.resource_pack.is_none() {
            bail!("Must specify at least one of `behaviorPack` or `resourcePack` in 'packs'")
        }
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        write_json("./config.json", self)
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_behavior_pack(&self) -> Option<PathBuf> {
        Some(PathBuf::from(&self.packs.behavior_pack.to_owned()?))
    }

    pub fn get_resource_pack(&self) -> Option<PathBuf> {
        Some(PathBuf::from(&self.packs.resource_pack.to_owned()?))
    }

    pub fn get_data_path(&self) -> PathBuf {
        PathBuf::from(&self.regolith.data_path)
    }

    pub fn get_profile(&self, profile_name: &str) -> Result<&Profile> {
        self.regolith
            .profiles
            .get(profile_name)
            .with_context(|| format!("Profile <profile>{profile_name}</> not found"))
    }

    pub fn get_filter(&self, filter_name: &str) -> Result<FilterDefinition> {
        let value = self
            .regolith
            .filter_definitions
            .get(filter_name)
            .with_context(|| {
                format!("Filter <filter>{filter_name}</> is not defined in filterDefinitions")
            })?
            .to_owned();
        FilterDefinition::from_value(value).map_err(|e| {
            anyhow!(
                "Invalid filter definition for <filter>{filter_name}</>\n\
                 <yellow> >></> {e}"
            )
        })
    }

    pub fn get_filters(&self) -> Result<BTreeMap<String, FilterDefinition>> {
        let mut filters = BTreeMap::<String, FilterDefinition>::new();
        for (name, value) in &self.regolith.filter_definitions {
            let filter = FilterDefinition::from_value(value.to_owned()).map_err(|e| {
                anyhow!(
                    "Invalid filter definition for <filter>{name}</>\n\
                     <yellow> >></> {e}"
                )
            })?;
            filters.insert(name.to_owned(), filter);
        }
        Ok(filters)
    }

    pub fn get_watcher(&self) -> Result<FileWatcher> {
        let mut watcher = FileWatcher::new()?;

        watcher.add_path("./config.json")?;
        if let Some(bp) = self.get_behavior_pack() {
            watcher.add_path(bp)?;
        }
        if let Some(rp) = self.get_resource_pack() {
            watcher.add_path(rp)?;
        }
        watcher.add_path(self.get_data_path())?;
        if let Some(watch_paths) = &self.regolith.watch_paths {
            for path in watch_paths {
                watcher.add_path(path)?;
            }
        }

        Ok(watcher)
    }
}

pub struct ConfigCst {
    root: CstRootNode,
    filter_definitions: CstObject,
    profiles: CstObject,
}

impl ConfigCst {
    pub fn load() -> Result<Self> {
        let data = std::fs::read_to_string("./config.json")?;
        let root = CstRootNode::parse(&data, &ParseOptions::default())?;
        let regolith = root.object_value_or_set().object_value_or_set("regolith");
        let filter_definitions = regolith.object_value_or_set("filterDefinitions");
        let profiles = regolith.object_value_or_set("profiles");
        Ok(Self {
            root,
            filter_definitions,
            profiles,
        })
    }

    pub fn save(&self) -> Result<()> {
        write_file("./config.json", self.root.to_string())?;
        Ok(())
    }

    pub fn add_filter(&self, filter_name: &str, remote: RemoteFilter) {
        let url = remote.url;
        let version = remote.version;
        let value = json!({ "url": url, "version": version });
        if let Some(definition) = self.filter_definitions.get(filter_name) {
            definition.set_value(value);
        } else {
            let index = self
                .filter_definitions
                .properties()
                .into_iter()
                .take_while(
                    |prop| match prop.name().and_then(|v| v.decoded_value().ok()) {
                        Some(prop) => filter_name.cmp(&prop) == std::cmp::Ordering::Greater,
                        None => false,
                    },
                )
                .count();
            self.filter_definitions.insert(index, filter_name, value);
        }
    }

    pub fn add_filter_to_profile(&self, filter_name: &str, profile_name: &str) -> bool {
        match self.profiles.object_value(profile_name) {
            Some(profile) => {
                let filters = profile.array_value_or_set("filters");
                let name = filter_name.to_owned();
                filters.append(json!({ "filter": name }));
                true
            }
            None => false,
        }
    }

    pub fn remove_filter(&self, filter_name: &str) -> bool {
        match self.filter_definitions.get(filter_name) {
            Some(definition) => {
                definition.remove();
                true
            }
            None => false,
        }
    }
}
