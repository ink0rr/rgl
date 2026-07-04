mod commands;
mod file_watcher;
mod fs;
mod logger;
mod rgl;

use anyhow::{Context, Result};
use clap::{crate_name, Parser, Subcommand};
use commands::*;
use enum_dispatch::enum_dispatch;
use logger::Logger;
use std::path::Path;
use std::thread;

fn main() {
    let cli = Cli::parse();
    Logger::set_debug(cli.debug);
    if let Err(e) = load_env_file(cli.env.as_deref()) {
        error!("{e}");
        e.chain().skip(1).for_each(|e| log!("<red>[+]</> {e}"));
        std::process::exit(1);
    }
    if let Err(e) = run_command(cli) {
        error!("{e}");
        e.chain().skip(1).for_each(|e| log!("<red>[+]</> {e}"));
        std::process::exit(1);
    }
}

/// Loads environment variables from a `.env` file, mirroring Regolith's behavior.
///
/// If `env` is `None`, defaults to `.env` in the current directory. Variables already
/// present in the process environment are not overwritten. If the file does not exist,
/// this is not an error and the function silently does nothing.
fn load_env_file(env: Option<&str>) -> Result<()> {
    let path = Path::new(env.filter(|e| !e.is_empty()).unwrap_or(".env"));
    if !path.exists() {
        return Ok(());
    }
    dotenvy::from_path(path).with_context(|| {
        format!(
            "Failed to load environment variables from file: {}",
            path.display()
        )
    })
}

fn run_command(cli: Cli) -> Result<()> {
    let cache_dir = rgl::get_cache_dir()?;
    if !cache_dir.exists() {
        fs::empty_dir(cache_dir)?;
    }
    let handle = match cli.subcommand {
        // Don't trigger update check when running these commands
        Subcommands::Upgrade(_) | Subcommands::Watch(_) => None,
        _ => Some(thread::spawn(rgl::version_check)),
    };
    measure_time!("Total time", {
        cli.subcommand
            .dispatch()
            .with_context(|| cli.subcommand.error_context())?;
    });
    if let Some(handle) = handle {
        match handle.join().unwrap() {
            Ok(version) => {
                if let Some(version) = version {
                    rgl::prompt_upgrade(version)?
                }
            }
            Err(e) => {
                warn!("Version check failed");
                e.chain().for_each(|e| log!("<yellow>[?]</> {e}"));
            }
        }
    }
    Ok(())
}

/// Fast and efficient Bedrock Addon Compiler
#[derive(Parser)]
#[command(bin_name = crate_name!(), version)]
struct Cli {
    #[command(subcommand)]
    subcommand: Subcommands,
    /// Print debug messages
    #[arg(long, global = true)]
    debug: bool,
    /// Path to a custom .env file to load
    #[arg(long, global = true)]
    env: Option<String>,
}

#[derive(Subcommand)]
#[enum_dispatch(Command)]
enum Subcommands {
    Add(Add),
    Apply(Apply),
    Clean(Clean),
    Exec(Exec),
    Get(Get),
    Info(Info),
    Init(Init),
    Install(Install),
    List(List),
    Remove(Remove),
    Run(Run),
    Uninstall(Uninstall),
    Update(Update),
    UpdateResolvers(UpdateResolvers),
    Upgrade(Upgrade),
    Watch(Watch),
}
