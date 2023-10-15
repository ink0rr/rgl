use super::{RglError, RglResult};
use std::env;

pub fn get_env(key: &str) -> RglResult<String> {
    env::var(key).map_err(|_| RglError::EnvironmentVariable {
        name: key.to_owned(),
    })
}
