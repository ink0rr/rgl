use super::get_current_dir;
use anyhow::{anyhow, bail, Context, Result};
use std::{ffi::OsStr, io, path::Path, process};

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

    pub fn current_dir(&mut self, dir: impl AsRef<Path>) -> &mut Self {
        self.command.current_dir(dir);
        self
    }

    pub fn setup_env(&mut self, filter_dir: impl AsRef<Path>) -> Result<&mut Self> {
        self.command.env("FILTER_DIR", filter_dir.as_ref());
        Ok(self)
    }

    pub fn run(&mut self) -> Result<process::Output> {
        let output = self
            .command
            .env("ROOT_DIR", get_current_dir()?)
            .spawn()
            .map_err(|_| anyhow!("Program {:?} not found", self.command.get_program()))
            .context("Failed spawning subprocess")?
            .wait_with_output()
            .context("Failed running subprocess")?;
        Ok(output)
    }

    pub fn run_silent(&mut self) -> Result<process::Output> {
        let output = self
            .command
            .env("ROOT_DIR", get_current_dir()?)
            .output()
            .map_err(|err| match err.kind() {
                io::ErrorKind::NotFound => {
                    anyhow!("Program {:?} not found", self.command.get_program())
                }
                _ => anyhow!(err),
            })
            .context("Failed running subprocess")?;
        if !output.status.success() {
            bail!("Process exited with non-zero status code");
        }
        Ok(output)
    }
}
