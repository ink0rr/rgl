use super::{
    get_filter_cache_dir, get_repo_cache_dir, resolve_url, Filter, FilterContext, LocalFilter,
    Subprocess,
};
use crate::fs::{copy_dir, empty_dir, read_json, rimraf};
use crate::info;
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
        for filter in config.filters {
            filter.run(context, temp, run_args)?;
        }
        Ok(())
    }
    fn install_dependencies(&self, context: &FilterContext) -> Result<()> {
        let config = RemoteFilterConfig::load(context)?;
        for filter in config.filters {
            filter.install_dependencies(context)?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct RemoteFilterConfig {
    pub filters: Vec<LocalFilter>,
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
        let name: String;
        let mut url: String;
        let version_arg: Option<String>;

        if arg.contains("==") {
            let parts: Vec<_> = arg.split("==").collect();
            if parts.len() != 2 {
                bail!("Invalid argument <b>{arg}</>");
            }
            url = parts[0].to_owned();
            version_arg = Some(parts[1].to_owned());
        } else {
            url = arg.to_owned();
            version_arg = None;
        }

        if url.contains('/') {
            let mut split: Vec<_> = url.split('/').collect();
            name = split.pop().unwrap().to_owned();
            url = split.join("/");
            if url.starts_with("https://") {
                url = url.replace("https://", "");
            }
        } else {
            name = url;
            url = resolve_url(&name)?;
        }

        let version = get_version(&name, &url, version_arg)?;
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
}

fn get_version(name: &str, url: &str, version_arg: Option<String>) -> Result<String> {
    if let Some(version) = &version_arg {
        if Version::parse(version).is_ok() {
            return Ok(version.to_owned());
        }
    }
    if version_arg.is_none() || version_arg == Some("latest".to_owned()) {
        let output = Subprocess::new("git")
            .args(["ls-remote", "--tags", &format!("https://{url}")])
            .run_silent()?;
        let output = String::from_utf8(output.stdout)?;

        let tag = output
            .trim()
            .split('\n')
            .filter_map(|line| {
                line.split(&format!("refs/tags/{name}-"))
                    .last()
                    .and_then(|version| Version::parse(version).ok())
            })
            .last();
        if let Some(tag) = tag {
            return Ok(tag.to_string());
        }
    }
    if version_arg.is_none() || version_arg == Some("HEAD".to_owned()) {
        let output = Subprocess::new("git")
            .args(["ls-remote", "--symref", &format!("https://{url}"), "HEAD"])
            .run_silent()?;
        let output = String::from_utf8(output.stdout)?;

        let sha = output
            .split('\n')
            .nth(1)
            .and_then(|line| line.split('\t').next());
        if let Some(sha) = sha {
            return Ok(sha.to_owned());
        }
    }
    bail!(
        "Failed to resolve filter version\n\
         <yellow> >></> Filter: {name}\n\
         <yellow> >></> URL: {url}\n\
         <yellow> >></> Version: {}",
        version_arg.unwrap_or("unspecified".to_owned())
    )
}
