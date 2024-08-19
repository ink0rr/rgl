use super::Command;
use crate::rgl::{get_cache_dir, get_global_filters_path, get_user_config_path};
use anyhow::Result;
use clap::{crate_version, Args};

#[derive(Args)]
pub struct Info {}
impl Command for Info {
    fn dispatch(&self) -> Result<()> {
        println!("rgl version: {}", crate_version!());
        println!("RGL_DIR location: {}", get_cache_dir()?.display());
        println!(
            "User config location: {}",
            get_user_config_path()?.display()
        );
        println!(
            "Global filters location: {}",
            get_global_filters_path()?.display()
        );
        Ok(())
    }
    fn error_context(&self) -> String {
        "Error getting rgl info".to_owned()
    }
}
