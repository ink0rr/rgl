use super::{Config, Eval, Export, Filter, FilterContext};
use crate::{debug, info, measure_time};
use anyhow::{bail, Context, Result};
use async_recursion::async_recursion;
use dashmap::DashSet;
use indexmap::IndexMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub export: Export,
    pub filters: Vec<ProfileEntry>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProfileEntry {
    Filter(FilterRunner),
    AsyncFilter {
        #[serde(rename = "asyncFilters")]
        async_filters: Vec<FilterRunner>,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterRunner {
    Filter {
        #[serde(rename = "filter")]
        filter_name: String,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        disabled: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        arguments: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        settings: Option<IndexMap<String, Value>>,
        #[serde(rename = "when", skip_serializing_if = "Option::is_none")]
        expression: Option<String>,
        #[serde(rename = "extraArguments", skip_serializing_if = "Option::is_none")]
        extra_arguments_mode: Option<String>,
    },
    ProfileFilter {
        #[serde(rename = "profile")]
        profile_name: String,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        disabled: bool,
    },
}

impl FilterRunner {
    fn get_name(&self) -> &str {
        match self {
            FilterRunner::Filter { filter_name, .. } => filter_name,
            FilterRunner::ProfileFilter { profile_name, .. } => profile_name,
        }
    }

    async fn run(
        &self,
        config: &Config,
        temp: &Path,
        root_profile: &str,
        extra_args: &[String],
    ) -> Result<DashSet<String>> {
        let export_data_names = DashSet::new();
        match self {
            FilterRunner::Filter {
                filter_name,
                disabled,
                arguments,
                settings,
                expression,
                extra_arguments_mode,
            } => {
                if *disabled {
                    info!("Filter <filter>{filter_name}</> is disabled, skipping.");
                    return Ok(export_data_names);
                }
                let filter = config.get_filter(filter_name)?;
                let mut run_args: Vec<String> = vec![];
                if let Some(settings) = settings {
                    run_args = vec![serde_json::to_string(settings)?]
                }
                let mut filter_arguments = arguments.to_owned().unwrap_or_default();
                match extra_arguments_mode.as_deref() {
                    None | Some("") | Some("ignore") => {}
                    Some("override") => filter_arguments = extra_args.to_vec(),
                    Some("append") => filter_arguments.extend(extra_args.iter().cloned()),
                    Some(mode) => bail!(
                        "The extraArguments property of a filter is invalid: {mode}\n\
                         Valid values: ignore, override, append"
                    ),
                }
                run_args.extend(filter_arguments);

                let context = FilterContext::new(filter_name, &filter)?;
                if let Some(expression) = expression {
                    let eval = Eval::new(root_profile, &context.filter_dir, settings.clone());
                    debug!("Evaluating expression: <d>{expression}</>");
                    if !eval.bool(expression).with_context(|| {
                        format!("Failed running evaluator for <filter>{filter_name}</>")
                    })? {
                        info!("Skipping filter <filter>{filter_name}</>");
                        return Ok(export_data_names);
                    }
                }
                info!("Running filter <filter>{filter_name}</>");
                filter
                    .run(&context, temp, &run_args)
                    .with_context(|| format!("Failed running filter <filter>{filter_name}</>"))?;
                if context.remote_config.is_some_and(|cfg| cfg.export_data) {
                    export_data_names.insert(filter_name.to_owned());
                }
                Ok(export_data_names)
            }
            FilterRunner::ProfileFilter {
                profile_name,
                disabled,
            } => {
                if *disabled {
                    info!("Filter <profile>{profile_name}</> is disabled, skipping.");
                    return Ok(export_data_names);
                }
                if profile_name == root_profile {
                    bail!("Found circular profile reference in <profile>{profile_name}</>");
                }
                let profile = config.get_profile(profile_name)?;
                info!("Running <profile>{profile_name}</> nested profile");
                // Extra CLI arguments are not forwarded into nested profiles,
                // matching Regolith's behavior.
                profile.run(config, temp, root_profile, &[]).await
            }
        }
    }
}

impl Profile {
    #[async_recursion]
    pub async fn run(
        &self,
        config: &Config,
        temp: &Path,
        root_profile: &str,
        extra_args: &[String],
    ) -> Result<DashSet<String>> {
        let mut export_data_names = DashSet::new();
        for entry in self.filters.iter() {
            match entry {
                ProfileEntry::Filter(filter) => {
                    measure_time!(filter.get_name(), {
                        export_data_names
                            .extend(filter.run(config, temp, root_profile, extra_args).await?);
                    });
                }
                ProfileEntry::AsyncFilter { async_filters } => {
                    let results: Vec<Result<DashSet<String>>> = async_filters
                        .par_iter()
                        .map(|entry| {
                            smol::block_on(entry.run(config, temp, root_profile, extra_args))
                        })
                        .collect();

                    for result in results {
                        export_data_names.extend(result?);
                    }
                }
            }
            for _ in 0..5 {
                smol::future::yield_now().await;
            }
        }
        Ok(export_data_names)
    }
}
