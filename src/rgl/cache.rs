use super::get_env;
use super::RglResult;
use md5;
use std::path::PathBuf;

#[cfg(any(target_os = "macos"))]
fn get_user_cache_dir() -> RglResult<PathBuf> {
    let home = get_env("HOME")?;
    Ok(PathBuf::from(home).join("Library").join("Caches"))
}

#[cfg(any(target_os = "windows"))]
fn get_user_cache_dir() -> RglResult<PathBuf> {
    let localappdata = get_env("LocalAppData")?;
    Ok(PathBuf::from(localappdata))
}

pub fn get_regolith_cache_dir() -> RglResult<PathBuf> {
    Ok(get_user_cache_dir()?.join("regolith"))
}

pub fn get_filter_cache_dir(https_url: &str) -> RglResult<PathBuf> {
    let digest = md5::compute(https_url.as_bytes());
    Ok(get_regolith_cache_dir()?
        .join("filter-cache")
        .join(format!("{digest:?}")))
}

pub fn get_resolver_cache_dir(https_url: &str) -> RglResult<PathBuf> {
    let digest = md5::compute(https_url.as_bytes());
    Ok(get_regolith_cache_dir()?
        .join("resolver-cache")
        .join(format!("{digest:?}")))
}
