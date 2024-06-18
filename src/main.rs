mod commands;
mod fs;
mod logger;
mod rgl;
mod watcher;

use anyhow::{Context, Result};
use clap::{crate_name, Parser, Subcommand};
use commands::*;
use enum_dispatch::enum_dispatch;
use logger::Logger;
use std::thread;

fn main() {
    let cli = Cli::parse();
    Logger::set_debug(cli.debug);
    if let Err(e) = run_command(cli) {
        error!("{}", e);
        e.chain().skip(1).for_each(|e| log!("<red>[+]</> {e}"));
        std::process::exit(1);
    }
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

/// Fast and minimal implementation of Regolith
#[derive(Parser)]
#[command(bin_name = crate_name!(), version)]
struct Cli {
    #[command(subcommand)]
    subcommand: Subcommands,
    /// Print debug messages
    #[arg(long, global = true)]
    debug: bool,
}

#[derive(Subcommand)]
#[enum_dispatch(Command)]
enum Subcommands {
    Add(Add),
    Apply(Apply),
    Clean(Clean),
    Exec(Exec),
    Get(Get),
    Init(Init),
    Install(Install),
    List(List),
    Remove(Remove),
    Run(Run),
    Uninstall(Uninstall),
    Upgrade(Upgrade),
    Watch(Watch),
}
