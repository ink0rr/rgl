use crate::info;
use crate::rgl::{FilterInstaller, FilterType};
use anyhow::Result;

pub fn install_tools(tools: Vec<&String>, force: bool) -> Result<()> {
    for arg in tools {
        info!("Installing tool <b>{}</>...", arg);
        let tool = FilterInstaller::from_arg(arg)?;
        if tool.install(FilterType::Tool, None, force)? {
            info!("Tool <b>{}</> successfully installed", tool.name);
        }
    }
    Ok(())
}
