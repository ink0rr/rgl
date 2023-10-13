use super::{empty_dir, get_resolver_cache_dir, read_json, RglError, RglResult, Subprocess};
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

fn get_resolver() -> RglResult<Resolver> {
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

pub fn resolve_url(name: &str) -> RglResult<String> {
    let resolver = get_resolver()?;
    match resolver.filters.get(name) {
        Some(data) => Ok(data.url.to_owned()),
        None => Err(RglError::FilterResolveFailed {
            filter_name: name.to_owned(),
        }),
    }
}
