use super::RemoteFilter;
use anyhow::Result;
use once_cell::sync::OnceCell;
use std::env;
use std::path::PathBuf;

pub fn get_current_dir() -> Result<PathBuf> {
    static CURRENT_DIR: OnceCell<PathBuf> = OnceCell::new();
    let current_dir = CURRENT_DIR.get_or_try_init(env::current_dir);
    Ok(current_dir?.to_owned())
}

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

pub fn get_cache_dir() -> Result<PathBuf> {
    static CACHE_DIR: OnceCell<PathBuf> = OnceCell::new();
    let cache_dir = CACHE_DIR.get_or_try_init(|| get_user_cache_dir().map(|path| path.join("rgl")));
    Ok(cache_dir?.to_owned())
}

pub fn get_timestamp_path() -> Result<PathBuf> {
    Ok(get_cache_dir()?.join("latest.txt"))
}

pub fn get_user_config_path() -> Result<PathBuf> {
    Ok(get_cache_dir()?.join("user_config.json"))
}

pub fn get_global_filters_path() -> Result<PathBuf> {
    Ok(get_cache_dir()?.join("global_filters.json"))
}

pub fn get_filter_cache_dir(name: &str, remote: &RemoteFilter) -> Result<PathBuf> {
    Ok(get_cache_dir()?
        .join("filters")
        .join(&remote.url)
        .join(name)
        .join(&remote.version))
}

pub fn get_repo_cache_dir() -> Result<PathBuf> {
    Ok(get_cache_dir()?.join("repo"))
}

pub fn get_resolver_cache_dir() -> Result<PathBuf> {
    Ok(get_cache_dir()?.join("resolver"))
}
