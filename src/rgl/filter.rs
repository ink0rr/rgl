use super::{FilterDeno, FilterGo, FilterNode, FilterPython, FilterRemote};
use anyhow::{bail, Context, Result};
use dunce::canonicalize;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub trait Filter {
    fn run(&self, temp: &Path, run_args: &[String]) -> Result<()>;
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

pub struct FilterArgs {
    pub name: String,
    pub script: PathBuf,
    pub filter_dir: PathBuf,
}

impl FilterDefinition {
    pub fn to_filter(&self, name: &str, filter_dir: Option<PathBuf>) -> Result<Box<dyn Filter>> {
        let inner = || {
            let filter: Box<dyn Filter> = match self {
                FilterDefinition::Local(def) => {
                    let args = FilterArgs::new(name, filter_dir, def)?;
                    match def.run_with.as_str() {
                        "deno" => Box::new(FilterDeno(args)),
                        "go" => Box::new(FilterGo(args)),
                        "nodejs" => Box::new(FilterNode(args)),
                        "python" => Box::new(FilterPython(args)),
                        filter_type => bail!("Filter type <b>{filter_type}</> not supported"),
                    }
                }
                FilterDefinition::Remote(def) => {
                    let filter_remote = FilterRemote::new(name)?;
                    let is_latest = matches!(def.version.as_str(), "HEAD" | "latest");
                    if !is_latest && def.version != filter_remote.version {
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
        };
        inner().context(format!("Invalid filter definition for <b>{name}</>"))
    }
}

impl FilterArgs {
    fn new(name: &str, filter_dir: Option<PathBuf>, def: &LocalFilterDefinition) -> Result<Self> {
        let script =
            canonicalize(&def.script).context(format!("Failed to resolve path {}", def.script))?;
        let filter_dir = filter_dir.unwrap_or_else(|| {
            script
                .parent()
                .map(|path| path.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."))
        });
        Ok(Self {
            name: name.to_owned(),
            script,
            filter_dir,
        })
    }
}
