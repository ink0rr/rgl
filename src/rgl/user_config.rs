use super::get_user_config_path;
use crate::fs::{read_json, write_json};
use crate::warn;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(default = "default_username")]
    pub username: String,
    #[serde(default = "default_resolvers")]
    pub resolvers: Vec<String>,
    pub mojang_dir: Option<String>,
    pub nodejs_runtime: Option<String>,
    pub nodejs_package_manager: Option<String>,
    pub python_command: Option<String>,
}

impl UserConfig {
    fn default() -> Self {
        Self {
            username: default_username(),
            resolvers: default_resolvers(),
            mojang_dir: None,
            nodejs_runtime: None,
            nodejs_package_manager: None,
            python_command: None,
        }
    }

    pub fn username() -> String {
        get_user_config().username.to_owned()
    }

    pub fn resolvers() -> Vec<String> {
        get_user_config().resolvers.to_owned()
    }

    pub fn mojang_dir() -> Option<String> {
        get_user_config().mojang_dir.to_owned()
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
}

fn default_username() -> String {
    "Your name".to_owned()
}

fn default_resolvers() -> Vec<String> {
    vec!["github.com/Bedrock-OSS/regolith-filter-resolver/resolver.json".to_owned()]
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
        match read_json(&path) {
            Ok(user_config) => user_config,
            Err(_) => {
                warn!("Failed to load user config, creating a new one...");
                let user_config = UserConfig::default();
                if let Err(e) = write_json(path, &user_config) {
                    warn!("Failed to write default user config: {}", e);
                }
                user_config
            }
        }
    })
}
