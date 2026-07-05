use super::{RemoteFilter, UserConfig};
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
    let cache_dir = CACHE_DIR.get_or_try_init(|| -> Result<PathBuf> {
        env::var("RGL_DIR")
            .map(PathBuf::from)
            .or_else(|_| get_user_cache_dir().map(|dir| dir.join("rgl")))
    });
    Ok(cache_dir?.to_owned())
}

/// Directory (inside the shared cache dir) that holds one subdirectory per
/// project when `use_project_app_data_storage` is enabled. Mirrors Regolith's
/// `appDataProjectCachePath`.
pub fn get_project_cache_dir() -> Result<PathBuf> {
    Ok(get_cache_dir()?.join("project-cache"))
}

/// Returns the absolute path to the project's `.regolith` directory.
///
/// By default this is `<project>/.regolith`. When the user config option
/// `use_project_app_data_storage` is enabled, it instead points to a
/// directory inside the shared cache dir, keyed by a hash of the absolute
/// project path, mirroring Regolith's `GetDotRegolith`/`getAppDataDotRegolith`.
pub fn get_dot_regolith_dir() -> Result<PathBuf> {
    if !UserConfig::use_project_app_data_storage() {
        return Ok(get_current_dir()?.join(".regolith"));
    }
    get_app_data_dot_regolith_dir()
}

/// Returns the absolute path to the project's `.regolith` directory inside
/// the shared cache dir, keyed by a hash of the absolute project path,
/// regardless of the `use_project_app_data_storage` setting. Mirrors
/// Regolith's `getAppDataDotRegolith`.
///
/// Used by `rgl clean`, which (like Regolith's `CleanCurrentProject`) always
/// clears both possible `.regolith` locations, so switching
/// `use_project_app_data_storage` doesn't leave stale caches behind.
pub fn get_app_data_dot_regolith_dir() -> Result<PathBuf> {
    let project_dir = get_current_dir()?;
    let hash = format!(
        "{:x}",
        md5::compute(project_dir.to_string_lossy().as_bytes())
    );
    Ok(get_project_cache_dir()?.join(hash))
}

pub fn get_user_config_path() -> Result<PathBuf> {
    Ok(get_cache_dir()?.join("user_config.json"))
}

pub fn get_global_filters_path() -> Result<PathBuf> {
    Ok(get_cache_dir()?.join("global_filters.json"))
}

/// Root directory (inside the shared cache dir) that holds all installed
/// filter caches, keyed by remote URL, filter name and version. Mirrors the
/// tree Regolith's `CleanFilterCache` wipes wholesale.
pub fn get_filters_cache_dir() -> Result<PathBuf> {
    Ok(get_cache_dir()?.join("filters"))
}

pub fn get_filter_cache_dir(name: &str, remote: &RemoteFilter) -> Result<PathBuf> {
    Ok(get_filters_cache_dir()?
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
