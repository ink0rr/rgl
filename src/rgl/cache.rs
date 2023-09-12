use md5;
use std::path::PathBuf;

#[cfg(any(target_os = "windows"))]
fn get_user_cache_dir() -> PathBuf {
    PathBuf::from(std::env!("LocalAppData").to_string())
}

pub fn get_regolith_cache_dir() -> PathBuf {
    get_user_cache_dir().join("regolith")
}

pub fn get_filter_cache_dir(https_url: &str) -> PathBuf {
    let digest = md5::compute(https_url.as_bytes());
    get_regolith_cache_dir()
        .join("filter-cache")
        .join(format!("{digest:?}"))
}

pub fn get_resolver_cache_dir(https_url: &str) -> PathBuf {
    let digest = md5::compute(https_url.as_bytes());
    get_regolith_cache_dir()
        .join("resolver-cache")
        .join(format!("{digest:?}"))
}
