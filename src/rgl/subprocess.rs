use super::get_current_dir;
use anyhow::{anyhow, bail, Context, Result};
use std::{
    ffi::OsStr,
    io::{self, BufRead, BufReader},
    path::Path,
    process,
    thread,
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

    pub fn current_dir(&mut self, dir: impl AsRef<Path>) -> &mut Self {
        self.command.current_dir(dir);
        self
    }

    pub fn setup_env(&mut self, filter_dir: impl AsRef<Path>) -> &mut Self {
        self.command.env("FILTER_DIR", filter_dir.as_ref());
        self
    }

    pub fn run(&mut self) -> Result<process::Output> {
        let output = self
            .command
            .env("ROOT_DIR", get_current_dir()?)
            .spawn()
            .map_err(|err| match err.kind() {
                io::ErrorKind::NotFound => self.program_not_found_error(),
                _ => anyhow!(err),
            })
            .context("Failed spawning subprocess")?
            .wait_with_output()
            .context("Failed running subprocess")?;
        if !output.status.success() {
            bail!("Process exited with non-zero status code");
        }
        Ok(output)
    }

    pub fn run_with_prefix(&mut self, prefix: &str) -> Result<process::Output> {
        let mut child = self
            .command
            .env("ROOT_DIR", get_current_dir()?)
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()
            .map_err(|err| match err.kind() {
                io::ErrorKind::NotFound => self.program_not_found_error(),
                _ => anyhow!(err),
            })
            .context("Failed spawning subprocess")?;

        let stdout = child.stdout.take().expect("stdout piped");
        let stderr = child.stderr.take().expect("stderr piped");
        let prefix_out = prefix.to_owned();
        let prefix_err = prefix.to_owned();

        let stdout_thread = thread::spawn(move || {
            BufReader::new(stdout).lines().for_each(|line| {
                if let Ok(line) = line {
                    crate::logger::Logger::info(format!("[{prefix_out}] {line}"));
                }
            });
        });
        let stderr_thread = thread::spawn(move || {
            BufReader::new(stderr).lines().for_each(|line| {
                if let Ok(line) = line {
                    crate::logger::Logger::info(format!("[{prefix_err}] {line}"));
                }
            });
        });

        let status = child.wait().context("Failed running subprocess")?;
        stdout_thread.join().ok();
        stderr_thread.join().ok();

        if !status.success() {
            bail!("Process exited with non-zero status code");
        }
        Ok(process::Output {
            status,
            stdout: vec![],
            stderr: vec![],
        })
    }

    pub fn run_silent(&mut self) -> Result<process::Output> {
        let output = self
            .command
            .env("ROOT_DIR", get_current_dir()?)
            .stderr(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .spawn()
            .map_err(|err| match err.kind() {
                io::ErrorKind::NotFound => self.program_not_found_error(),
                _ => anyhow!(err),
            })
            .context("Failed spawning subprocess")?
            .wait_with_output()
            .context("Failed running subprocess")?;
        if !output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stderr));
            bail!("Process exited with non-zero status code");
        }
        Ok(output)
    }

    fn program_not_found_error(&self) -> anyhow::Error {
        let program = self.command.get_program();
        let mut message = format!("Program {:?} not found", program);
        let install_link = match program.to_str() {
            Some("bun") => Some("https://bun.sh/docs/installation"),
            Some("deno") => Some("https://docs.deno.com/runtime/#install-deno"),
            Some("git") => Some("https://git-scm.com/downloads"),
            Some("go") => Some("https://go.dev/doc/install"),
            Some("node") => Some("https://nodejs.org/en/download/prebuilt-installer"),
            Some("python") => Some("https://www.python.org/downloads"),
            _ => None,
        };
        if let Some(link) = install_link {
            message.push_str(&format!(". Install it from {link}"));
        }
        anyhow!(message)
    }
}
