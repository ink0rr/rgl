use super::{get_global_filters_path, RemoteFilter};
use crate::fs::{read_json, write_json};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{btree_map::Iter, BTreeMap};

#[derive(Default, Serialize, Deserialize)]
pub struct GlobalFilters {
    filters: BTreeMap<String, RemoteFilter>,
}

impl GlobalFilters {
    pub fn load() -> Result<Self> {
        let path = get_global_filters_path()?;
        if path.exists() {
            return read_json(path);
        }
        let global_filters = GlobalFilters::default();
        global_filters.save()?;
        Ok(global_filters)
    }

    pub fn save(&self) -> Result<()> {
        write_json(get_global_filters_path()?, self)
    }

    pub fn get(&self, name: &str) -> Result<RemoteFilter> {
        let filter = self
            .filters
            .get(name)
            .with_context(|| format!("Filter <filter>{name}</> is not installed"))?
            .to_owned();
        Ok(filter)
    }

    pub fn iter(&self) -> Iter<'_, String, RemoteFilter> {
        self.filters.iter()
    }

    pub fn add(&mut self, name: &str, filter: RemoteFilter) -> Option<RemoteFilter> {
        self.filters.insert(name.to_owned(), filter)
    }

    pub fn remove(&mut self, name: &str) -> Option<RemoteFilter> {
        self.filters.remove(name)
    }
}
