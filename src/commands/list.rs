use super::Command;
use crate::log;
use crate::rgl::{Config, FilterDefinition, GlobalFilters};
use anyhow::{Context, Result};
use clap::Args;

/// List filters defined in the `config.json` file
#[derive(Args)]
#[clap(alias = "ls")]
pub struct List {
    #[arg(short, long)]
    global: bool,
}

impl Command for List {
    fn dispatch(&self) -> Result<()> {
        match self.global {
            false => list_project(),
            true => list_global(),
        }
    }
    fn error_context(&self) -> String {
        "Error listing installed filters".to_owned()
    }
}

fn list_project() -> Result<()> {
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

fn list_global() -> Result<()> {
    let global_filters = GlobalFilters::load()?;

    let mut filters = vec![];
    for (name, filter) in global_filters.iter() {
        filters.push((name.to_owned(), filter.version.to_owned()));
    }
    print("Global filters:", &filters);
    Ok(())
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
