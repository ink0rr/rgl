use anyhow::{Context, Result};
use dunce::canonicalize;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process,
};

pub struct Subprocess {
    command: process::Command,
}

impl Subprocess {
    pub fn new<S>(command: S) -> Self
    where
        S: AsRef<OsStr>,
    {
        Self {
            command: process::Command::new(command),
        }
    }

    pub fn arg<S>(&mut self, arg: S) -> &mut Self
    where
        S: AsRef<OsStr>,
    {
        self.command.arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }

    pub fn current_dir<P>(&mut self, dir: P) -> &mut Self
    where
        P: AsRef<Path>,
    {
        self.command.current_dir(dir);
        self
    }

    pub fn setup_env(&mut self, filter_dir: &PathBuf) -> Result<&mut Self> {
        let root_dir = canonicalize(".")?;
        let filter_dir = canonicalize(filter_dir)?;
        self.command
            .env("ROOT_DIR", root_dir)
            .env("FILTER_DIR", filter_dir);
        Ok(self)
    }

    pub fn run(&mut self) -> Result<process::Output> {
        self.command
            .spawn()
            .context("Failed spawning subprocess")?
            .wait_with_output()
            .context("Failed running subprocess")
    }

    pub fn run_silent(&mut self) -> Result<process::Output> {
        self.command.output().context("Failed running subprocess")
    }
}
