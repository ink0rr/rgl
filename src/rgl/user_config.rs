use super::get_user_config_path;
use crate::fs::{read_json, write_json};
use crate::warn;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(default = "default_username")]
    pub username: String,
    #[serde(default = "default_resolvers")]
    pub resolvers: Vec<String>,
    #[serde(default = "default_resolver_update_interval")]
    pub resolver_update_interval: u64,
    #[serde(default = "default_filter_cache_update_cooldown")]
    pub filter_cache_update_cooldown: u64,
    #[serde(default = "default_websocket_port")]
    pub websocket_port: u16,
    #[serde(default)]
    pub force_compat: bool,
    #[serde(default)]
    pub subprocess_logging: bool,
    #[serde(default)]
    pub use_project_app_data_storage: bool,
    pub mojang_dir: Option<String>,
    pub nodejs_runtime: Option<String>,
    pub nodejs_package_manager: Option<String>,
    pub python_command: Option<String>,
    pub tmp_dir: Option<String>,
    /// Overrides which JS runtime family (`bun`, `deno` or `node`) executes
    /// `nodejs` filters, keyed by filter name, with `"*"` matching every
    /// `nodejs` filter that doesn't have its own entry. Mirrors Go's
    /// `UserConfig.NodeRunnerOverride`.
    pub node_runner_override: Option<IndexMap<String, String>>,
    /// Explicit path/binary overrides for each filter runner, matching Go's
    /// `UserConfig.*Runner` fields. Unlike `nodejs_runtime`/`python_command`
    /// (which additionally act as informal runtime switches), these only
    /// override the executable used to run/install a filter of the matching
    /// kind.
    pub bun_runner: Option<String>,
    pub deno_runner: Option<String>,
    pub dotnet_runner: Option<String>,
    pub java_runner: Option<String>,
    pub nim_runner: Option<String>,
    pub nimble_runner: Option<String>,
    pub node_runner: Option<String>,
    pub npm_runner: Option<String>,
    pub python_runner: Option<String>,
}

impl UserConfig {
    fn default() -> Self {
        Self {
            username: default_username(),
            resolvers: default_resolvers(),
            resolver_update_interval: default_resolver_update_interval(),
            filter_cache_update_cooldown: default_filter_cache_update_cooldown(),
            websocket_port: default_websocket_port(),
            force_compat: false,
            subprocess_logging: false,
            use_project_app_data_storage: false,
            mojang_dir: None,
            nodejs_runtime: None,
            nodejs_package_manager: None,
            python_command: None,
            tmp_dir: None,
            node_runner_override: None,
            bun_runner: None,
            deno_runner: None,
            dotnet_runner: None,
            java_runner: None,
            nim_runner: None,
            nimble_runner: None,
            node_runner: None,
            npm_runner: None,
            python_runner: None,
        }
    }

    pub fn username() -> String {
        get_user_config().username.to_owned()
    }

    pub fn resolvers() -> Vec<String> {
        get_user_config().resolvers.to_owned()
    }

    pub fn resolver_update_interval() -> u64 {
        get_user_config().resolver_update_interval
    }

    pub fn filter_cache_update_cooldown() -> u64 {
        get_user_config().filter_cache_update_cooldown
    }

    pub fn websocket_port() -> u16 {
        get_user_config().websocket_port
    }

    pub fn force_compat() -> bool {
        get_user_config().force_compat
    }

    pub fn subprocess_logging() -> bool {
        get_user_config().subprocess_logging
    }

    pub fn use_project_app_data_storage() -> bool {
        get_user_config().use_project_app_data_storage
    }

    pub fn mojang_dir() -> Option<String> {
        get_user_config().mojang_dir.to_owned()
    }

    pub fn tmp_dir() -> Option<String> {
        get_user_config().tmp_dir.to_owned()
    }

    pub fn nodejs_runtime() -> String {
        get_user_config()
            .nodejs_runtime
            .to_owned()
            .unwrap_or("node".to_owned())
    }

    pub fn nodejs_package_manager() -> String {
        get_user_config()
            .nodejs_package_manager
            .to_owned()
            .unwrap_or(match cfg!(windows) {
                true => "npm.cmd".to_owned(),
                false => "npm".to_owned(),
            })
    }

    pub fn python_command() -> String {
        get_user_config()
            .python_command
            .to_owned()
            .unwrap_or("python".to_owned())
    }

    /// The JS runtime family override (`"bun"`, `"deno"` or `"node"`) for a
    /// `nodejs` filter, looked up by filter name and falling back to the
    /// `"*"` entry. Mirrors Go's `NodeRunnerOverride` lookup in
    /// `FilterInstallerFromObject`, where a filter-specific entry takes
    /// precedence over `"*"`.
    pub fn node_runner_override(filter_name: &str) -> Option<String> {
        let map = get_user_config().node_runner_override.as_ref()?;
        map.get(filter_name).or_else(|| map.get("*")).cloned()
    }

    pub fn bun_runner() -> String {
        get_user_config()
            .bun_runner
            .to_owned()
            .unwrap_or("bun".to_owned())
    }

    pub fn deno_runner() -> String {
        get_user_config()
            .deno_runner
            .to_owned()
            .unwrap_or("deno".to_owned())
    }

    pub fn dotnet_runner() -> String {
        get_user_config()
            .dotnet_runner
            .to_owned()
            .unwrap_or("dotnet".to_owned())
    }

    pub fn java_runner() -> String {
        get_user_config()
            .java_runner
            .to_owned()
            .unwrap_or("java".to_owned())
    }

    pub fn nim_runner() -> String {
        get_user_config()
            .nim_runner
            .to_owned()
            .unwrap_or("nim".to_owned())
    }

    pub fn nimble_runner() -> String {
        get_user_config()
            .nimble_runner
            .to_owned()
            .unwrap_or("nimble".to_owned())
    }

    /// The binary used to run `nodejs` filters whose resolved runtime
    /// family is `node` (i.e. no `node_runner_override` applies). Prefers
    /// the explicit `node_runner` path, falling back to the legacy
    /// `nodejs_runtime` field (which historically also doubled as an
    /// informal runtime switch), then `"node"`.
    pub fn node_runner() -> String {
        get_user_config()
            .node_runner
            .to_owned()
            .unwrap_or_else(Self::nodejs_runtime)
    }

    /// The binary used to install dependencies for `nodejs` filters whose
    /// resolved runtime family is `node`. Prefers the explicit `npm_runner`
    /// path, falling back to `nodejs_package_manager`.
    pub fn npm_runner() -> String {
        get_user_config()
            .npm_runner
            .to_owned()
            .unwrap_or_else(Self::nodejs_package_manager)
    }

    /// The binary used to run `python` filters. Prefers the explicit
    /// `python_runner` path, falling back to the legacy `python_command`
    /// field, then `"python"`.
    pub fn python_runner() -> String {
        get_user_config()
            .python_runner
            .to_owned()
            .unwrap_or_else(Self::python_command)
    }
}

fn default_username() -> String {
    "Your name".to_owned()
}

fn default_resolvers() -> Vec<String> {
    vec!["github.com/Bedrock-OSS/regolith-filter-resolver/resolver.json".to_owned()]
}

fn default_resolver_update_interval() -> u64 {
    300
}

fn default_filter_cache_update_cooldown() -> u64 {
    300
}

fn default_websocket_port() -> u16 {
    80
}

fn get_user_config() -> &'static UserConfig {
    static USER_CONFIG: OnceLock<UserConfig> = OnceLock::new();
    USER_CONFIG.get_or_init(|| {
        let path = get_user_config_path();
        if path.is_err() {
            warn!("Failed to get user config path");
            return UserConfig::default();
        }
        let path = path.unwrap();
        read_json(&path).unwrap_or_else(|_| {
            warn!("Failed to load user config, creating a new one...");
            let user_config = UserConfig::default();
            if let Err(e) = write_json(path, &user_config) {
                warn!("Failed to write default user config: {e}");
            }
            user_config
        })
    })
}
