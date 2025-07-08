use super::{get_resolver_cache_dir, Subprocess, UserConfig};
use crate::debug;
use crate::fs::{empty_dir, is_dir_empty, read_json, set_modified_time};
use anyhow::{bail, Context, Result};
use once_cell::sync::OnceCell;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Default, Serialize, Deserialize)]
pub struct Resolver {
    filters: HashMap<String, ResolverData>,
}

#[derive(Serialize, Deserialize)]
struct ResolverData {
    url: String,
    versions: Option<Vec<String>>,
}

impl Resolver {
    fn get(name: &str) -> Result<&ResolverData> {
        get_resolver()
            .context("Failed to load filter resolver")?
            .filters
            .get(name)
            .context(format!("Failed to resolve filter <b>{name}</>"))
    }

    pub fn resolve_url(name: &str) -> Result<String> {
        Self::get(name).map(|data| data.url.to_owned())
    }

    pub fn resolve_version(name: &str, url: &str, version_arg: Option<String>) -> Result<String> {
        // Try to get version from resolver
        let get_version = || -> Option<String> {
            let data = Self::get(name).ok()?;
            if data.url != url || data.versions.is_none() {
                return None;
            };
            let mut versions = data.versions.to_owned().unwrap();
            versions.sort_by_key(|v| Version::parse(v).ok());
            match &version_arg {
                Some(arg) if versions[1..].contains(arg) => Some(arg.to_owned()),
                None => versions[1..].last().cloned(),
                _ => None,
            }
        };
        if let Some(version) = get_version() {
            return Ok(version);
        }
        debug!("Using `git ls-remote` to resolve version");
        let https_url = format!("https://{url}");
        let version_arg = version_arg.as_deref();
        // Check if version is available in git tags
        if let Ok(version) = Version::parse(version_arg.unwrap_or_default()) {
            let tag = format!("{name}-{version}");
            let output = Subprocess::new("git")
                .args(["ls-remote", &https_url, &tag])
                .run_silent()
                .context(format!(
                    "Failed to check version from `{url}`. Is the url correct?"
                ))?;
            let output = String::from_utf8(output.stdout)?;
            if output.split('\n').any(|line| line.ends_with(&tag)) {
                return Ok(version.to_string());
            }
        }
        if version_arg.is_none() || version_arg == Some("latest") {
            let output = Subprocess::new("git")
                .args(["ls-remote", "--tags", &https_url])
                .run_silent()
                .context(format!(
                    "Failed to get latest version from `{url}`. Is the url correct?"
                ))?;
            let output = String::from_utf8(output.stdout)?;
            let mut versions: Vec<Version> = output
                .split('\n')
                .filter_map(|line| {
                    line.split(&format!("refs/tags/{name}-"))
                        .last()
                        .and_then(|version| Version::parse(version).ok())
                })
                .collect();
            versions.sort();
            if let Some(version) = versions.last() {
                return Ok(version.to_string());
            }
        }
        if version_arg.is_none() || version_arg == Some("HEAD") {
            let output = Subprocess::new("git")
                .args(["ls-remote", "--symref", &https_url, "HEAD"])
                .run_silent()
                .context(format!(
                    "Failed to get HEAD version from `{url}`. Is the url correct?"
                ))?;
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
            version_arg.unwrap_or("unspecified")
        )
    }
}

fn get_resolver() -> Result<&'static Resolver> {
    static RESOLVER: OnceCell<Resolver> = OnceCell::new();
    RESOLVER.get_or_try_init(|| {
        let mut resolver = Resolver::default();
        for resolver_url in UserConfig::resolvers() {
            let (url, path) = parse_resolver_url(&resolver_url)
                .context(format!("Failed to parse url `{resolver_url}`",))?;
            let resolver_dir = get_resolver_cache_dir()?.join(&url);
            let resolver_file = resolver_dir.join(&path);
            let https_url = format!("https://{url}");
            if is_dir_empty(&resolver_dir)? {
                empty_dir(&resolver_dir)?;
                Subprocess::new("git")
                    .args(["clone", &https_url, "."])
                    .current_dir(&resolver_dir)
                    .run_silent()
                    .context(format!("Failed to clone `{https_url}`"))?;
            } else {
                let last_modified = resolver_file.metadata()?.modified()?.elapsed()?.as_secs();
                if last_modified > UserConfig::resolver_update_interval() {
                    Subprocess::new("git")
                        .args(["pull"])
                        .current_dir(&resolver_dir)
                        .run_silent()
                        .context(format!("Failed to pull `{https_url}`"))?;
                    set_modified_time(&resolver_file, SystemTime::now())?;
                }
            }
            let data = read_json::<Resolver>(resolver_file)?;
            resolver.filters.extend(data.filters)
        }
        Ok(resolver)
    })
}

fn parse_resolver_url(url: &str) -> Result<(String, String)> {
    let url_parts: Vec<&str> = url.split('/').collect();
    if url.starts_with("https://") || url_parts.len() < 4 {
        bail!("Incorrect URL format. Expected: `github.com/<user>/<repo>/<resolver-file-path>`");
    }
    let repo_url = url_parts[0..3].join("/");
    let path = url_parts[3..].join("/");
    Ok((repo_url, path))
}
