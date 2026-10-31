use super::RemoteFilter;
use anyhow::{bail, Result};
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
    match env::var("XDG_CACHE_HOME") {
        Ok(dir) if !dir.is_empty() => {
            let dir = PathBuf::from(dir);
            if !dir.is_absolute() {
                bail!("Path in $XDG_CACHE_HOME is relative");
            }
            Ok(dir)
        }
        _ => match env::var("HOME") {
            Ok(home) if !home.is_empty() => Ok(PathBuf::from(home).join(".cache")),
            _ => bail!("Neither $XDG_CACHE_HOME nor $HOME are defined"),
        },
    }
}

#[cfg(target_os = "macos")]
fn get_user_cache_dir() -> Result<PathBuf> {
    match env::var("HOME") {
        Ok(home) if !home.is_empty() => Ok(PathBuf::from(home).join("Library").join("Caches")),
        _ => bail!("$HOME is not defined"),
    }
}

#[cfg(target_os = "windows")]
fn get_user_cache_dir() -> Result<PathBuf> {
    match env::var("LocalAppData") {
        Ok(dir) if !dir.is_empty() => Ok(PathBuf::from(dir)),
        _ => bail!("%LocalAppData% is not defined"),
    }
}

#[cfg(target_os = "linux")]
fn get_user_config_dir() -> Result<PathBuf> {
    match env::var("XDG_CONFIG_HOME") {
        Ok(dir) if !dir.is_empty() => {
            let dir = PathBuf::from(dir);
            if !dir.is_absolute() {
                bail!("Path in $XDG_CONFIG_HOME is relative");
            }
            Ok(dir)
        }
        _ => match env::var("HOME") {
            Ok(home) if !home.is_empty() => Ok(PathBuf::from(home).join(".config")),
            _ => bail!("Neither $XDG_CONFIG_HOME nor $HOME are defined"),
        },
    }
}

#[cfg(target_os = "macos")]
fn get_user_config_dir() -> Result<PathBuf> {
    match env::var("HOME") {
        Ok(home) if !home.is_empty() => Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")),
        _ => bail!("$HOME is not defined"),
    }
}

#[cfg(target_os = "windows")]
fn get_user_config_dir() -> Result<PathBuf> {
    match env::var("AppData") {
        Ok(dir) if !dir.is_empty() => Ok(PathBuf::from(dir)),
        _ => bail!("%AppData% is not defined"),
    }
}

pub fn get_cache_dir() -> Result<PathBuf> {
    static CACHE_DIR: OnceCell<PathBuf> = OnceCell::new();
    let cache_dir = CACHE_DIR.get_or_try_init(|| -> Result<PathBuf> {
        env::var("RGL_CACHE_DIR")
            .map(PathBuf::from)
            .or_else(|_| get_user_cache_dir().map(|dir| dir.join("rgl")))
    });
    Ok(cache_dir?.to_owned())
}

pub fn get_config_dir() -> Result<PathBuf> {
    static CONFIG_DIR: OnceCell<PathBuf> = OnceCell::new();
    let config_dir = CONFIG_DIR.get_or_try_init(|| -> Result<PathBuf> {
        env::var("RGL_CONFIG_DIR")
            .map(PathBuf::from)
            .or_else(|_| get_user_config_dir().map(|dir| dir.join("rgl")))
    });
    Ok(config_dir?.to_owned())
}

pub fn get_user_config_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("user_config.json"))
}

pub fn get_global_filters_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("global_filters.json"))
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
