use anyhow::{Context, Result};
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use std::{path::Path, sync::mpsc, time::Duration};

pub struct Watcher {
    rx: mpsc::Receiver<notify::Result<Vec<DebouncedEvent>>>,
    debouncer: Debouncer<RecommendedWatcher>,
}

impl Watcher {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let debouncer = new_debouncer(Duration::from_millis(100), tx)
            .context("Failed to create file watcher")?;
        Ok(Self { rx, debouncer })
    }

    pub fn watch(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        self.debouncer
            .watcher()
            .watch(path, RecursiveMode::Recursive)
            .context(format!(
                "Failed to watch directory\n\
                 <yellow> >></> Path: {}",
                path.display()
            ))
    }

    pub fn wait_changes(&self) {
        self.rx.iter().next();
    }
}
