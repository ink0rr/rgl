use super::{Config, Eval, Export, Filter, FilterContext};
use crate::{debug, info, measure_time};
use anyhow::{bail, Context, Result};
use async_recursion::async_recursion;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashSet, path::Path};

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
    #[async_recursion]
    pub async fn run(
        &self,
        config: &Config,
        temp: &Path,
        root_profile: &str,
    ) -> Result<HashSet<String>> {
        let mut export_data_names = HashSet::new();
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
                            let eval =
                                Eval::new(root_profile, &context.filter_dir, settings.clone());
                            debug!("Evaluating expression: <d>{expression}</>");
                            if !eval.bool(expression).with_context(|| {
                                format!("Failed running evaluator for <filter>{filter_name}</>")
                            })? {
                                info!("Skipping filter <filter>{filter_name}</>");
                                continue;
                            }
                        }
                        info!("Running filter <filter>{filter_name}</>");
                        filter.run(&context, temp, &run_args).with_context(|| {
                            format!("Failed running filter <filter>{filter_name}</>")
                        })?;
                        if context.remote_config.is_some_and(|cfg| cfg.export_data) {
                            export_data_names.insert(filter_name.to_owned());
                        }
                    });
                }
                FilterRunner::ProfileFilter { profile_name } => {
                    if profile_name == root_profile {
                        bail!("Found circular profile reference in <profile>{profile_name}</>");
                    }
                    let profile = config.get_profile(profile_name)?;

                    info!("Running <profile>{profile_name}</> nested profile");
                    export_data_names.extend(profile.run(config, temp, root_profile).await?);
                }
            }
            for _ in 0..5 {
                smol::future::yield_now().await;
            }
        }
        Ok(export_data_names)
    }
}
