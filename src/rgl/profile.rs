use super::{Config, Export, Filter, FilterContext};
use crate::{debug, info, measure_time, rgl::FilterEvaluator};
use anyhow::{bail, Context, Result};
use clap::crate_version;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub export: Export,
    pub filters: Vec<FilterRunner>,
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
        #[serde(rename = "when", skip_serializing_if = "Option::is_none")]
        expression: Option<String>,
    },
    ProfileFilter {
        #[serde(rename = "profile")]
        profile_name: String,
    },
}

impl Profile {
    pub fn run(&self, config: &Config, temp: &Path, root_profile: &str) -> Result<()> {
        for entry in self.filters.iter() {
            match entry {
                FilterRunner::Filter {
                    filter_name,
                    arguments,
                    settings,
                    expression,
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
                        let context = FilterContext::new(filter_name, &filter)?;
                        if let Some(expression) = expression {
                            let evaluator = FilterEvaluator::new(
                                crate_version!(),
                                root_profile,
                                &context.filter_dir,
                                settings,
                            );
                            debug!("Evaluating expression <b>{expression}</>");
                            if !evaluator.run(expression).with_context(|| {
                                format!("Failed running evaluator for <b>{filter_name}</>")
                            })? {
                                info!("Skipping filter <b>{filter_name}</>");
                                continue;
                            }
                        }
                        info!("Running filter <b>{filter_name}</>");
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
}
