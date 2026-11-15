use super::Command;
use crate::Cli;
use anyhow::Result;
use clap::{crate_name, Args, CommandFactory};
use clap_complete::{generate, Shell};

/// Generate a shell completion script and print it to stdout
#[derive(Args)]
pub struct Completions {
    /// The shell to generate completions for
    #[arg(value_enum)]
    shell: Shell,
}

impl Command for Completions {
    fn dispatch(&self) -> Result<()> {
        let mut cmd = Cli::command();
        generate(self.shell, &mut cmd, crate_name!(), &mut std::io::stdout());
        Ok(())
    }
    fn error_context(&self) -> String {
        "Error generating shell completions".to_owned()
    }
}
