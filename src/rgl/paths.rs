use anyhow::Result;
use std::env;
use std::path::PathBuf;

#[cfg(target_os = "linux")]
fn get_user_cache_dir() -> Result<PathBuf> {
    let home = env::var("HOME")?;
    Ok(PathBuf::from(home).join(".cache"))
}

#[cfg(target_os = "macos")]
fn get_user_cache_dir() -> Result<PathBuf> {
    let home = env::var("HOME")?;
    Ok(PathBuf::from(home).join("Library").join("Caches"))
}

#[cfg(target_os = "windows")]
fn get_user_cache_dir() -> Result<PathBuf> {
    let localappdata = env::var("LocalAppData")?;
    Ok(PathBuf::from(localappdata))
}

pub fn get_regolith_cache_dir() -> Result<PathBuf> {
    Ok(get_user_cache_dir()?.join("regolith"))
}

pub fn get_filter_cache_dir(https_url: &str) -> Result<PathBuf> {
    let digest = md5::compute(https_url.as_bytes());
    Ok(get_regolith_cache_dir()?
        .join("filter-cache")
        .join(format!("{digest:?}")))
}

pub fn get_resolver_cache_dir(https_url: &str) -> Result<PathBuf> {
    let digest = md5::compute(https_url.as_bytes());
    Ok(get_regolith_cache_dir()?
        .join("resolver-cache")
        .join(format!("{digest:?}")))
}
