use super::{RglError, RglResult};
use notify::{Error, RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use simplelog::error;
use std::{path::Path, sync::mpsc::Receiver, time::Duration};

pub struct FileWatcher {
    rx: Receiver<std::result::Result<Vec<DebouncedEvent>, Vec<Error>>>,
    debouncer: Debouncer<RecommendedWatcher>,
}

impl FileWatcher {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let debouncer = new_debouncer(Duration::from_millis(100), None, tx).unwrap_or_else(|err| {
            error!("Failed to create file watcher: {}", err);
            std::process::exit(1);
        });
        Self { rx, debouncer }
    }

    pub fn watch(&mut self, path: &str) -> RglResult<()> {
        match self
            .debouncer
            .watcher()
            .watch(Path::new(path), RecursiveMode::Recursive)
        {
            Ok(_) => Ok(()),
            Err(cause) => Err(RglError::WatchError {
                path: path.to_owned(),
                cause: RglError::WrapError(cause.into()).into(),
            }),
        }
    }

    pub fn wait_changes(&self) {
        self.rx.iter().next();
    }
}
