use super::{FilterDeno, FilterNode, FilterPython, FilterRemote, RglError, RglResult};
use dunce::canonicalize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub trait Filter {
    fn run(&self, temp: &PathBuf, run_args: &Vec<String>) -> RglResult<()>;
    fn install_dependencies(&self) -> RglResult<()> {
        Ok(())
    }
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
    pub run_with: String,
    pub script: String,
}

#[derive(Serialize, Deserialize)]
pub struct RemoteFilterDefinition {
    pub url: String,
    pub version: String,
}

impl FilterDefinition {
    pub fn to_filter(&self, name: &str, filter_dir: Option<PathBuf>) -> RglResult<Box<dyn Filter>> {
        let filter: Box<dyn Filter> = match self {
            FilterDefinition::Local(def) => {
                let filter_dir = filter_dir.unwrap_or_else(|| PathBuf::from("."));
                let script =
                    canonicalize(&def.script).map_err(|_| RglError::InvalidFilterDefinition {
                        filter_name: name.to_owned(),
                        cause: RglError::PathNotExists {
                            path: def.script.to_owned(),
                        }
                        .into(),
                    })?;
                match def.run_with.as_str() {
                    "deno" => Box::new(FilterDeno::new(filter_dir, script)),
                    "nodejs" => Box::new(FilterNode::new(filter_dir, script)),
                    "python" => Box::new(FilterPython::new(filter_dir, script)),
                    filter_type => {
                        return Err(RglError::FilterTypeNotSupported {
                            filter_type: filter_type.to_owned(),
                        })
                    }
                }
            }
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
