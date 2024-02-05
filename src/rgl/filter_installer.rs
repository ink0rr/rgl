use super::{
    get_filter_cache_dir, resolve_url, Filter, FilterContext, FilterType, RemoteFilterConfig,
};
use crate::fs::{copy_dir, empty_dir, move_dir, rimraf};
use crate::subprocess::Subprocess;
use crate::{info, warn};
use anyhow::{bail, Result};
use semver::Version;
use std::path::Path;

pub struct FilterInstaller {
    pub name: String,
    pub url: String,
    pub git_ref: String,
}

impl FilterInstaller {
    pub fn new(name: &str, url: String, git_ref: String) -> Self {
        Self {
            name: name.to_owned(),
            url,
            git_ref,
        }
    }

    pub fn from_arg(arg: &str) -> Result<Self> {
        let name: String;
        let mut url: String;
        let version: Option<String>;

        if arg.contains("==") {
            let parts: Vec<_> = arg.split("==").collect();
            if parts.len() != 2 {
                bail!("Invalid argument <b>{arg}</>");
            }
            url = parts[0].to_owned();
            version = Some(parts[1].to_owned());
        } else {
            url = arg.to_owned();
            version = None;
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

        let git_ref = get_git_ref(&name, &url, version)?;
        info!("Resolved <b>{arg}</> to <b>{url}@{git_ref}</>");

        Ok(Self { name, url, git_ref })
    }

    pub fn install(
        &self,
        filter_type: FilterType,
        data_path: Option<&Path>,
        force: bool,
    ) -> Result<bool> {
        let name = &self.name;
        let filter_dir = filter_type.cache_dir(name)?;
        if filter_dir.exists() && !force {
            warn!("Filter {name} already added, use --force to overwrite");
            return Ok(false);
        } else {
            rimraf(&filter_dir)?;
        }

        let https_url = format!("https://{}", self.url);
        let cache_dir = get_filter_cache_dir(&https_url)?;

        if cache_dir.exists() {
            Subprocess::new("git")
                .args(vec!["fetch", "--all"])
                .current_dir(&cache_dir)
                .run_silent()?;
        } else {
            empty_dir(&cache_dir)?;
            Subprocess::new("git")
                .args(vec!["clone", &https_url, "."])
                .current_dir(&cache_dir)
                .run_silent()?;
        }

        Subprocess::new("git")
            .args(vec!["checkout", &self.git_ref])
            .current_dir(&cache_dir)
            .run_silent()?;

        copy_dir(cache_dir.join(name), &filter_dir)?;

        if let Some(data_path) = data_path {
            let filter_data = filter_dir.join("data");
            let target_path = data_path.join(name);
            if filter_data.is_dir() && !target_path.exists() {
                info!("Moving filter data to <b>{}</>", target_path.display());
                move_dir(filter_data, target_path)?;
            }
        }

        let config = RemoteFilterConfig::new(filter_dir, &self.git_ref)?;
        let context = FilterContext::new(filter_type, name)?;
        for filter in config.filters {
            info!("Installing dependencies for <b>{name}</>...");
            filter.install_dependencies(&context)?;
        }
        Ok(true)
    }
}

fn get_git_ref(name: &str, url: &str, version: Option<String>) -> Result<String> {
    if let Some(version) = &version {
        if let Ok(version) = Version::parse(version) {
            return Ok(format!("{name}-{version}"));
        }
    }
    if version.is_none() || version == Some("latest".to_owned()) {
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
                    .map(|version| format!("{name}-{version}"))
            })
            .last();
        if let Some(tag) = tag {
            return Ok(tag);
        }
    }
    if version.is_none() || version == Some("HEAD".to_owned()) {
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
        version.unwrap_or("latest".to_owned())
    )
}

pub fn ref_to_version(git_ref: &str) -> String {
    git_ref
        .split('-')
        .nth(1)
        .and_then(|version| Version::parse(version).ok())
        .map(|version| version.to_string())
        .unwrap_or(git_ref.to_owned())
}
