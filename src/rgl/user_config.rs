use super::get_user_config_path;
use crate::fs::{read_json, write_json};
use crate::warn;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::OnceLock;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeRunner {
    Nodejs,
    Bun,
    Deno,
}

#[derive(Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(default = "default_username")]
    pub username: String,
    #[serde(default = "default_resolvers")]
    pub resolvers: Vec<String>,
    #[serde(default = "default_resolver_update_interval")]
    pub resolver_update_interval: u64,
    #[serde(default = "default_websocket_port")]
    pub websocket_port: u16,
    #[serde(default)]
    pub force_compat: bool,
    #[serde(default)]
    pub subprocess_logging: bool,
    /// Maps a filter name, or `*` for every filter, to the runtime used for `nodejs` filters.
    #[serde(default)]
    pub node_runner_override: BTreeMap<String, NodeRunner>,
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
            websocket_port: default_websocket_port(),
            force_compat: false,
            subprocess_logging: false,
            node_runner_override: BTreeMap::new(),
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

    pub fn websocket_port() -> u16 {
        get_user_config().websocket_port
    }

    pub fn force_compat() -> bool {
        get_user_config().force_compat
    }

    pub fn subprocess_logging() -> bool {
        get_user_config().subprocess_logging
    }

    /// Runtime for the `nodejs` filter `name`. A filter-specific entry wins over `*`.
    pub fn node_runner_override(name: &str) -> NodeRunner {
        let overrides = &get_user_config().node_runner_override;
        overrides
            .get(name)
            .or_else(|| overrides.get("*"))
            .copied()
            .unwrap_or(NodeRunner::Nodejs)
    }

    pub fn bun_runner() -> &'static str {
        get_user_config().bun_runner.as_deref().unwrap_or("bun")
    }

    pub fn deno_runner() -> &'static str {
        get_user_config().deno_runner.as_deref().unwrap_or("deno")
    }

    pub fn dotnet_runner() -> &'static str {
        get_user_config()
            .dotnet_runner
            .as_deref()
            .unwrap_or("dotnet")
    }

    pub fn java_runner() -> &'static str {
        get_user_config().java_runner.as_deref().unwrap_or("java")
    }

    pub fn nim_runner() -> &'static str {
        get_user_config().nim_runner.as_deref().unwrap_or("nim")
    }

    pub fn nimble_runner() -> &'static str {
        get_user_config()
            .nimble_runner
            .as_deref()
            .unwrap_or("nimble")
    }

    pub fn node_runner() -> &'static str {
        get_user_config().node_runner.as_deref().unwrap_or("node")
    }

    pub fn npm_runner() -> &'static str {
        get_user_config()
            .npm_runner
            .as_deref()
            .unwrap_or(match cfg!(windows) {
                true => "npm.cmd",
                false => "npm",
            })
    }

    pub fn python_runner() -> &'static str {
        get_user_config()
            .python_runner
            .as_deref()
            .unwrap_or("python")
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
        if path.exists() {
            return read_json(&path).unwrap_or_else(|e| {
                warn!("Failed to load user config, using defaults: {e:?}");
                UserConfig::default()
            });
        }
        let user_config = UserConfig::default();
        if let Err(e) = write_json(path, &user_config) {
            warn!("Failed to write default user config: {e}");
        }
        user_config
    })
}
