use super::{FilterDeno, FilterNode, FilterRemote, RglError, RglResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub trait Filter {
    fn install_dependencies(&self) -> RglResult<()> {
        Ok(())
    }
    fn run(&mut self, temp: &PathBuf, run_args: &Vec<String>) -> RglResult<()>;
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterDefinition {
    Local(LocalFilterDefinition),
    Remote(RemoteFilterDefinition),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalFilterDefinition {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    pub run_with: String,
    pub script: String,
}

#[derive(Serialize, Deserialize)]
pub struct RemoteFilterDefinition {
    pub url: String,
    pub version: String,
}

impl FilterDefinition {
    pub fn to_filter(&self, name: &str) -> RglResult<Box<dyn Filter>> {
        let filter: Box<dyn Filter> = match self {
            FilterDefinition::Local(def) => match def.run_with.as_str() {
                "deno" => Box::new(FilterDeno::new(name, &def.script)),
                "nodejs" => Box::new(FilterNode::new(name, &def.script)),
                filter_type => {
                    return Err(RglError::FilterTypeNotSupported {
                        filter_type: filter_type.to_owned(),
                    })
                }
            },
            FilterDefinition::Remote(def) => {
                let filter_remote = FilterRemote::new(name)?;
                if def.version != "HEAD"
                    && def.version != "latest"
                    && def.version != filter_remote.version
                {
                    return Err(RglError::FilterVersionMismatch {
                        filter_name: name.to_owned(),
                        installed_version: filter_remote.version,
                        required_version: def.version.to_owned(),
                    });
                }
                Box::new(filter_remote)
            }
        };
        Ok(filter)
    }
}
