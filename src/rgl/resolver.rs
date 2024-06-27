use super::{get_resolver_cache_dir, Subprocess, UserConfig};
use crate::fs::{empty_dir, read_json, set_modified_time};
use anyhow::{bail, Context, Result};
use once_cell::sync::OnceCell;
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
}

impl Resolver {
    pub fn resolve(name: &str) -> Result<String> {
        get_resolver()?
            .filters
            .get(name)
            .map(|data| data.url.to_owned())
            .context(format!("Failed to resolve filter <b>{name}</>"))
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
            if resolver_dir.exists() {
                let last_modified = resolver_file.metadata()?.modified()?.elapsed()?.as_secs();
                if last_modified > UserConfig::resolver_update_interval() {
                    Subprocess::new("git")
                        .args(["pull"])
                        .current_dir(&resolver_dir)
                        .run_silent()?;
                    set_modified_time(&resolver_file, SystemTime::now())?;
                }
            } else {
                empty_dir(&resolver_dir)?;
                Subprocess::new("git")
                    .args(["clone", &format!("https://{url}"), "."])
                    .current_dir(&resolver_dir)
                    .run_silent()?;
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
