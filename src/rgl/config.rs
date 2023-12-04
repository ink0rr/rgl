use super::{Export, FilterRunner, Profile};
use crate::fs::{read_json, write_json};
use anyhow::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub author: String,
    pub name: String,
    pub packs: Packs,
    pub regolith: Regolith,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Packs {
    pub behavior_pack: String,
    pub resource_pack: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Regolith {
    pub data_path: String,
    pub filter_definitions: BTreeMap<String, Value>,
    pub profiles: IndexMap<String, Profile>,
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
            schema: "https://raw.githubusercontent.com/Bedrock-OSS/regolith-schemas/main/config/v1.1.json".to_owned(),
            author: "Your name".to_owned(),
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

    pub fn load() -> Result<Config> {
        read_json::<Config>("./config.json")
    }

    pub fn save(&self) -> Result<()> {
        write_json("./config.json", self)
    }
}
