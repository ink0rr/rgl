use super::{read_json, FilterDefinition, Profile, RglError, RglResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Serialize, Deserialize)]
pub struct FilterRunner {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<HashMap<String, Value>>,
}

pub fn get_config() -> RglResult<Config> {
    match read_json::<Config>("./config.json") {
        Ok(config) => Ok(config),
        Err(e) => Err(RglError::Config { cause: e.into() }),
    }
}
