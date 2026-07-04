use super::Command;
use crate::info;
use crate::rgl::Resolver;
use anyhow::Result;
use clap::Args;

/// Force-update the cached resolver repositories, bypassing the update cooldown
#[derive(Args)]
pub struct UpdateResolvers;

impl Command for UpdateResolvers {
    fn dispatch(&self) -> Result<()> {
        info!("Updating resolvers...");
        Resolver::refresh()?;
        info!("Resolvers successfully updated!");
        Ok(())
    }
    fn error_context(&self) -> String {
        "Error updating resolvers".to_owned()
    }
}
