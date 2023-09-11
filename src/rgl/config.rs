use super::{read_json, write_json, FilterDefinition, Profile, RglError, RglResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub filter_definitions: HashMap<String, FilterDefinition>,
    pub profiles: HashMap<String, Profile>,
}

impl Config {
    pub fn load() -> RglResult<Config> {
        match read_json::<Config>("./config.json") {
            Ok(config) => Ok(config),
            Err(e) => Err(RglError::Config { cause: e.into() }),
        }
    }
    pub fn save(&self) -> RglResult<()> {
        write_json("./config.json", self)
    }
}
