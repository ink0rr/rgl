use super::{Result, RglError};
use std::{ffi::OsStr, path::Path, process};

pub struct Subprocess {
    command: process::Command,
}

impl Subprocess {
    pub fn new(command: &str) -> Self {
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

    pub fn run(&mut self) -> Result<process::Output> {
        match self.command.spawn() {
            Ok(handler) => match handler.wait_with_output() {
                Ok(output) => Ok(output),
                Err(e) => Err(RglError::SubprocessError(e.to_string())),
            },
            Err(e) => Err(RglError::SubprocessError(e.to_string())),
        }
    }
}
