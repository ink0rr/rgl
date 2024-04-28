use super::Command;
use crate::rgl::{Config, FilterDefinition};
use anyhow::{Context, Result};
use clap::Args;
use paris::log;

/// List filters defined in the `config.json` file
#[derive(Args)]
#[clap(alias = "ls")]
pub struct List;

impl Command for List {
    fn dispatch(&self) -> Result<()> {
        let config = Config::load()?;

        let mut local_filters = vec![];
        let mut remote_filters = vec![];
        for (name, value) in config.get_filters() {
            let filter = FilterDefinition::from_value(value.to_owned())?;
            match filter {
                FilterDefinition::Local(_) => {
                    let run_with = value["runWith"]
                        .as_str()
                        .context("Invalid filter definition")?
                        .to_owned();
                    local_filters.push((name, run_with));
                }
                FilterDefinition::Remote(filter) => {
                    remote_filters.push((name, filter.version));
                }
            }
        }
        print("Local filters:", &local_filters);
        print("Remote filters:", &remote_filters);
        Ok(())
    }
    fn error_context(&self) -> String {
        "Error listing installed filters".to_owned()
    }
}

fn print(label: &str, filters: &Vec<(String, String)>) {
    if filters.is_empty() {
        return;
    }
    log!("<green>{label}</>");
    for (name, info) in filters {
        log!("  {name} <cyan>{info}</>");
    }
}
