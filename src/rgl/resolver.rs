use super::get_resolver_cache_dir;
use crate::fs::{empty_dir, read_json};
use crate::subprocess::Subprocess;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct Resolver {
    filters: HashMap<String, ResolverData>,
}

#[derive(Serialize, Deserialize)]
struct ResolverData {
    url: String,
}

fn get_resolver() -> Result<Resolver> {
    // TODO: Get resolvers from user config
    let resolver_url = "https://github.com/Bedrock-OSS/regolith-filter-resolver";
    let cache = get_resolver_cache_dir(resolver_url)?;
    if !cache.is_dir() {
        empty_dir(&cache)?;
        Subprocess::new("git")
            .args(["clone", resolver_url, "."])
            .current_dir(&cache)
            .run_silent()?;
    }

    Subprocess::new("git")
        .args(["pull"])
        .current_dir(&cache)
        .run_silent()?;

    read_json::<Resolver>(cache.join("resolver.json"))
}

pub fn resolve_url(name: &str) -> Result<String> {
    get_resolver()
        .context("Failed getting filter resolver")?
        .filters
        .get(name)
        .map(|data| data.url.to_owned())
        .context(format!("Failed to resolve filter <b>{name}</>"))
}
