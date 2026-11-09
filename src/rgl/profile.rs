use super::{get_current_dir, Eval, ExportTargets, Filter, FilterInfo, RunContext};
use crate::{debug, info, measure_time};
use anyhow::{bail, Context, Result};
use async_recursion::async_recursion;
use dashmap::DashSet;
use indexmap::IndexMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub export: ExportTargets,
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
        #[serde(rename = "when", skip_serializing_if = "Option::is_none")]
        expression: Option<String>,
    },
}

impl FilterRunner {
    fn get_name(&self) -> &str {
        match self {
            FilterRunner::Filter { filter_name, .. } => filter_name,
            FilterRunner::ProfileFilter { profile_name, .. } => profile_name,
        }
    }

    async fn run(&self, ctx: &RunContext<'_>) -> Result<DashSet<String>> {
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
                let filter = ctx.config.get_filter(filter_name)?;
                let mut filter_arguments = arguments.to_owned().unwrap_or_default();
                match extra_arguments_mode.as_deref() {
                    None | Some("") | Some("ignore") => {}
                    Some("override") => filter_arguments = ctx.extra_args.to_vec(),
                    Some("append") => filter_arguments.extend(ctx.extra_args.iter().cloned()),
                    Some(mode) => bail!(
                        "The extraArguments property of a filter is invalid: {mode}\n\
                         Valid values: ignore, override, append"
                    ),
                }
                let info = FilterInfo::new(filter_name, &filter)?;
                let ctx = &ctx.with_settings(settings.as_ref());
                if let Some(expression) = expression {
                    let eval = Eval::new(ctx, &info.filter_dir);
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
                    .run(ctx, &info, &filter_arguments)
                    .with_context(|| format!("Failed running filter <filter>{filter_name}</>"))?;
                if info.remote_config.is_some_and(|cfg| cfg.export_data) {
                    export_data_names.insert(filter_name.to_owned());
                }
                Ok(export_data_names)
            }
            FilterRunner::ProfileFilter {
                profile_name,
                disabled,
                expression,
            } => {
                if *disabled {
                    info!("Filter <profile>{profile_name}</> is disabled, skipping.");
                    return Ok(export_data_names);
                }
                if let Some(expression) = expression {
                    let eval = Eval::new(ctx, &get_current_dir()?);
                    debug!("Evaluating expression: <d>{expression}</>");
                    if !eval.bool(expression).with_context(|| {
                        format!("Failed running evaluator for <profile>{profile_name}</>")
                    })? {
                        info!("Skipping nested profile <profile>{profile_name}</>");
                        return Ok(export_data_names);
                    }
                }
                let profile = ctx.config.get_profile(profile_name)?;
                let nested_ctx = ctx.nested_profile(profile_name)?;
                info!("Running <profile>{profile_name}</> nested profile");
                profile.run(&nested_ctx).await
            }
        }
    }
}

impl Profile {
    #[async_recursion]
    pub async fn run(&self, ctx: &RunContext<'_>) -> Result<DashSet<String>> {
        let mut export_data_names = DashSet::new();
        for entry in self.filters.iter() {
            match entry {
                ProfileEntry::Filter(filter) => {
                    measure_time!(filter.get_name(), {
                        export_data_names.extend(filter.run(ctx).await?);
                    });
                }
                ProfileEntry::AsyncFilter { async_filters } => {
                    let results: Vec<Result<DashSet<String>>> = async_filters
                        .par_iter()
                        .map(|entry| smol::block_on(entry.run(ctx)))
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
