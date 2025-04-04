use super::{
    get_filter_cache_dir, get_repo_cache_dir, Filter, FilterContext, FilterEvaluator, LocalFilter,
    Resolver, Subprocess,
};
use crate::fs::{copy_dir, empty_dir, read_json, rimraf};
use crate::{debug, info, warn};
use anyhow::{bail, Context, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Serialize, Deserialize)]
pub struct RemoteFilter {
    pub url: String,
    pub version: String,
}

impl Filter for RemoteFilter {
    fn run(&self, context: &FilterContext, temp: &Path, run_args: &[String]) -> Result<()> {
        let config = RemoteFilterConfig::load(context)?;
        for data in config.filters {
            if let Some(expression) = &data.expression {
                let name = &context.name;
                let evaluator = FilterEvaluator::new(name, &context.filter_dir, &None);
                debug!("Evaluating expression <b>{expression}</>");
                if !evaluator
                    .run(expression)
                    .with_context(|| format!("Failed running evaluator for <b>{name}</>"))?
                {
                    continue;
                }
            }
            // This behavior is different from Regolith. It might break some filters
            // that need the arguments to be passed in a specific order.
            let mut run_args = run_args.to_vec();
            if let Some(arguments) = data.arguments {
                run_args.extend(arguments);
            };
            data.filter.run(context, temp, &run_args)?;
        }
        Ok(())
    }
    fn install_dependencies(&self, context: &FilterContext) -> Result<()> {
        let config = RemoteFilterConfig::load(context)?;
        for data in config.filters {
            data.filter.install_dependencies(context)?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct RemoteFilterConfig {
    pub filters: Vec<RemoteFilterConfigData>,
}

#[derive(Serialize, Deserialize)]
pub struct RemoteFilterConfigData {
    pub arguments: Option<Vec<String>>,
    #[serde(rename = "when", skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,
    #[serde(flatten)]
    pub filter: LocalFilter,
}

impl RemoteFilterConfig {
    fn load(context: &FilterContext) -> Result<Self> {
        read_json(context.filter_dir.join("filter.json")).context(format!(
            "Failed to load config for filter <b>{}</>",
            context.name
        ))
    }
}

impl RemoteFilter {
    /// Parse RemoteFilter from string argument
    pub fn parse(arg: &str) -> Result<(String, Self)> {
        // Extract version argument if present
        let parts: Vec<_> = arg.split('@').collect();
        let (arg, version_arg) = match parts.len() {
            1 => (parts[0], None),
            2 => (parts[0], Some(parts[1].to_owned())),
            _ => bail!("Invalid argument <b>{arg}</>"),
        };

        // Resolve filter name and URL
        let url_parts: Vec<_> = arg.split('/').collect();
        let (name, url) = match url_parts.len() {
            1 => (arg.to_owned(), Resolver::resolve_url(arg)?),
            4 => (url_parts[3].to_owned(), url_parts[..3].join("/")),
            _ => bail!("Incorrect URL format. Expected: `github.com/<user>/<repo>/<filter-name>`"),
        };

        let version = Resolver::resolve_version(&name, &url, version_arg)?;
        info!("Resolved <b>{arg}</> to <b>{url}/{name}@{version}</>");

        Ok((name, Self { url, version }))
    }

    pub fn install(&self, name: &str, data_path: Option<&Path>, force: bool) -> Result<()> {
        let url = &self.url;
        let version = &self.version;
        let filter_dir = get_filter_cache_dir(name, self)?;

        if force {
            rimraf(&filter_dir)?;
        }
        if !filter_dir.exists() {
            let repo_dir = get_repo_cache_dir()?.join(url);
            if repo_dir.exists() {
                Subprocess::new("git")
                    .args(vec!["fetch", "--all"])
                    .current_dir(&repo_dir)
                    .run_silent()?;
            } else {
                empty_dir(&repo_dir)?;
                Subprocess::new("git")
                    .args(vec!["clone", &format!("https://{url}"), "."])
                    .current_dir(&repo_dir)
                    .run_silent()
                    .context(format!("Failed to clone `{url}`"))?;
            }
            let git_ref = Version::parse(version)
                .map(|_| format!("{name}-{version}"))
                .unwrap_or(version.to_owned());
            Subprocess::new("git")
                .args(vec!["checkout", &git_ref])
                .current_dir(&repo_dir)
                .run_silent()
                .context(format!("Failed to checkout `{git_ref}`"))?;
            copy_dir(repo_dir.join(name), &filter_dir)?;
        }
        if let Some(data_path) = data_path {
            let filter_data = filter_dir.join("data");
            let target_path = data_path.join(name);
            if filter_data.is_dir() && !target_path.exists() {
                info!("Copying filter data to <b>{}</>", target_path.display());
                copy_dir(filter_data, target_path)?;
            }
        }

        let filter = self.to_owned().into();
        let context = FilterContext::new(name, &filter)?;
        info!("Installing dependencies for <b>{name}</>...");
        filter.install_dependencies(&context)
    }

    pub fn update(&mut self, name: &str, data_path: Option<&Path>, force: bool) -> Result<()> {
        let current_version = self.version.to_owned();
        let latest_version = Resolver::resolve_version(name, &self.url, Some("latest".to_owned()))?;
        if current_version == latest_version {
            warn!("Filter <b>{name}</> is already up-to-date");
            return Ok(());
        }
        info!("Updating filter <b>{name}</> <cyan>{current_version}</> → <cyan>{latest_version}</>...");
        self.version = latest_version.to_owned();
        self.install(name, data_path, force)?;
        Ok(())
    }
}
