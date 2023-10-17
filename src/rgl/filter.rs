use super::{FilterDeno, FilterGo, FilterNode, FilterPython, FilterRemote};
use anyhow::{bail, Context, Result};
use dunce::canonicalize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub trait Filter {
    fn run(&self, temp: &PathBuf, run_args: &Vec<String>) -> Result<()>;
    fn install_dependencies(&self) -> Result<()> {
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
    fn to_filter_impl(&self, name: &str, filter_dir: Option<PathBuf>) -> Result<Box<dyn Filter>> {
        let filter: Box<dyn Filter> = match self {
            FilterDefinition::Local(def) => {
                let filter_dir = filter_dir.unwrap_or_else(|| PathBuf::from("."));
                let script = canonicalize(&def.script)
                    .context(format!("Failed to resolve path {}", def.script))?;

                match def.run_with.as_str() {
                    "deno" => Box::new(FilterDeno::new(filter_dir, script)),
                    "go" => Box::new(FilterGo::new(filter_dir, script)),
                    "nodejs" => Box::new(FilterNode::new(filter_dir, script)),
                    "python" => Box::new(FilterPython::new(filter_dir, script)),
                    filter_type => bail!("Filter type <b>{filter_type}</> not supported"),
                }
            }
            FilterDefinition::Remote(def) => {
                let filter_remote = FilterRemote::new(name)?;
                if def.version != "HEAD"
                    && def.version != "latest"
                    && def.version != filter_remote.version
                {
                    bail!(
                        "Filter version mismatch\n\
                         <yellow> >></> Filter: {}\n\
                         <yellow> >></> Installed version: {}\n\
                         <yellow> >></> Required version: {}",
                        name,
                        filter_remote.version,
                        def.version
                    );
                }
                Box::new(filter_remote)
            }
        };
        Ok(filter)
    }

    pub fn to_filter(&self, name: &str, filter_dir: Option<PathBuf>) -> Result<Box<dyn Filter>> {
        self.to_filter_impl(name, filter_dir)
            .context(format!("Invalid filter definition for <b>{name}</>"))
    }
}
