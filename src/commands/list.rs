use crate::rgl::{Config, FilterDefinition};
use anyhow::{Context, Result};
use paris::log;

pub fn list() -> Result<()> {
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

fn print(label: &str, filters: &Vec<(String, String)>) {
    if filters.is_empty() {
        return;
    }
    log!("<green>{label}</>");
    for (name, info) in filters {
        log!("  {name} <cyan>{info}</>");
    }
}
