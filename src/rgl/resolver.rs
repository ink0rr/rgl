use super::{get_resolver_cache_dir, Subprocess, UserConfig};
use crate::fs::{empty_dir, read_json};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize)]
struct Resolver {
    filters: HashMap<String, ResolverData>,
}

#[derive(Serialize, Deserialize)]
struct ResolverData {
    url: String,
}

fn get_resolver() -> Result<Resolver> {
    let mut result = Resolver::default();
    for resolver_url in UserConfig::resolvers() {
        let (url, path) = resolve_resolver_url(&resolver_url)
            .context(format!("Failed to resolve url `{resolver_url}`",))?;
        let resolver_dir = get_resolver_cache_dir()?.join(&url);
        if resolver_dir.exists() {
            Subprocess::new("git")
                .args(["pull"])
                .current_dir(&resolver_dir)
                .run_silent()?;
        } else {
            empty_dir(&resolver_dir)?;
            Subprocess::new("git")
                .args(["clone", &format!("https://{url}"), "."])
                .current_dir(&resolver_dir)
                .run_silent()?;
        }
        let resolver = read_json::<Resolver>(resolver_dir.join(path))?;
        result.filters.extend(resolver.filters)
    }
    Ok(result)
}

fn resolve_resolver_url(url: &str) -> Result<(String, String)> {
    let url_parts: Vec<&str> = url.split('/').collect();
    if url.starts_with("https://") || url_parts.len() < 4 {
        bail!("Incorrect URL format. Expected: `github.com/<user>/<repo>/<resolver-file-path>`");
    }
    let repo_url = url_parts[0..3].join("/");
    let path = url_parts[3..].join("/");
    Ok((repo_url, path))
}

pub fn resolve_url(name: &str) -> Result<String> {
    get_resolver()
        .context("Failed getting filter resolver")?
        .filters
        .get(name)
        .map(|data| data.url.to_owned())
        .context(format!("Failed to resolve filter <b>{name}</>"))
}
