use crate::fs::{empty_dir, try_symlink};
use crate::rgl::{Config, Session};
use crate::{info, measure_time};
use anyhow::Result;
use std::path::Path;

pub fn apply(profile_name: &str) -> Result<()> {
    let config = Config::load()?;
    let mut session = Session::lock()?;

    let temp = Path::new(".regolith").join("tmp");
    let temp_bp = temp.join("BP");
    let temp_rp = temp.join("RP");

    empty_dir(&temp)?;
    try_symlink(config.get_behavior_pack(), temp_bp)?;
    try_symlink(config.get_resource_pack(), temp_rp)?;
    try_symlink(config.get_data_path(), temp.join("data"))?;

    let profile = config.get_profile(profile_name)?;
    measure_time!(profile_name, {
        info!("Running <b>{profile_name}</> profile");
        profile.run(&config, &temp, profile_name)?;
    });
    info!("Successfully applied profile <b>{profile_name}</>");
    session.unlock()
}
