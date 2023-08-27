use super::{FilterDeno, FilterNode, FilterRemote, RglError, RglResult};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

pub trait Filter {
    fn run(&mut self, temp: &PathBuf, run_args: &Vec<String>) -> RglResult<()>;
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterDefinition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl FilterDefinition {
    pub fn to_filter(&self, name: &str) -> RglResult<Box<dyn Filter>> {
        match &self.run_with {
            Some(run_with) => match run_with.as_str() {
                "deno" => Ok(Box::new(self.to_filter_impl::<FilterDeno>(name)?)),
                "nodejs" => Ok(Box::new(self.to_filter_impl::<FilterNode>(name)?)),
                _ => Err(RglError::FilterTypeNotSupported {
                    filter_type: run_with.to_owned(),
                }),
            },
            None => Ok(Box::new(FilterRemote::new(&name)?)),
        }
    }

    fn to_filter_impl<T>(&self, name: &str) -> RglResult<T>
    where
        T: DeserializeOwned,
    {
        let mut value = json!(self);
        value["name"] = json!(name);
        match serde_json::from_value::<T>(value) {
            Ok(v) => Ok(v),
            Err(e) => Err(RglError::InvalidFilterDefinition {
                filter_name: name.to_owned(),
                cause: RglError::SerdeJson(e).into(),
            }),
        }
    }
}
