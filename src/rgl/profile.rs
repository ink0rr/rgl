use super::{find_mojang_dir, Config, Filter, FilterContext};
use crate::{info, measure_time};
use anyhow::{bail, Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub export: Export,
    pub filters: Vec<FilterRunner>,
}

#[derive(Serialize, Deserialize)]
pub struct Export {
    pub target: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterRunner {
    Filter {
        #[serde(rename = "filter")]
        filter_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        arguments: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        settings: Option<IndexMap<String, Value>>,
    },
    ProfileFilter {
        #[serde(rename = "profile")]
        profile_name: String,
    },
}

impl Profile {
    pub fn run(&self, config: &Config, temp: &PathBuf, root_profile: &str) -> Result<()> {
        for entry in self.filters.iter() {
            match entry {
                FilterRunner::Filter {
                    filter_name,
                    arguments,
                    settings,
                } => {
                    let filter = config.get_filter(filter_name)?;
                    let mut run_args: Vec<String> = vec![];
                    if let Some(settings) = settings {
                        run_args = vec![serde_json::to_string(settings)?]
                    }
                    if let Some(args) = arguments {
                        run_args.extend(args.iter().map(|x| x.to_owned()));
                    }

                    measure_time!(filter_name, {
                        info!("Running filter <b>{filter_name}</>");
                        let context = FilterContext::new(filter.get_type(), filter_name)?;
                        filter
                            .run(&context, temp, &run_args)
                            .context(format!("Failed running filter <b>{filter_name}</>"))?;
                    });
                }
                FilterRunner::ProfileFilter { profile_name } => {
                    if profile_name == root_profile {
                        bail!("Found circular profile reference in <b>{profile_name}</>");
                    }
                    let profile = config.get_profile(profile_name)?;

                    info!("Running <b>{profile_name}</> nested profile");
                    profile.run(config, temp, root_profile)?;
                }
            }
        }
        Ok(())
    }

    /// Returns bp, rp, and temp paths respectively.
    pub fn get_export_paths(&self, name: &str) -> Result<(PathBuf, PathBuf, PathBuf)> {
        let target = self.export.target.as_str();
        let (bp, rp) = match target {
            "development" => {
                let mojang_dir = find_mojang_dir()?;
                let bp = mojang_dir
                    .join("development_behavior_packs")
                    .join(format!("{}_bp", name));
                let rp = mojang_dir
                    .join("development_resource_packs")
                    .join(format!("{}_rp", name));
                (bp, rp)
            }
            "local" => {
                let build = PathBuf::from("build");
                if !build.exists() {
                    fs::create_dir(&build)?;
                }
                let bp = build.join("BP");
                let rp = build.join("RP");
                (bp, rp)
            }
            _ => bail!("Export target <b>{target}</> is not valid"),
        };
        let temp = match target {
            "development" => find_mojang_dir()?.join(".rgl").join(name),
            _ => PathBuf::from(".regolith").join("tmp"),
        };
        Ok((bp, rp, temp))
    }
}
