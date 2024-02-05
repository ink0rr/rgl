mod commands;
mod fs;
mod logger;
mod rgl;
mod subprocess;
mod watcher;

use anyhow::{Context, Result};
use clap::{crate_version, Arg, ArgAction, ArgMatches, Command};
use paris::log;
use std::{env, thread};

fn main() {
    let matches = cli().get_matches();
    logger::init(matches.get_flag("debug"));
    if let Err(e) = run_command(matches) {
        error!("{}", e);
        e.chain().skip(1).for_each(|e| log!("<red>[+]</> {e}"));
        std::process::exit(1);
    }
}

fn cli() -> Command {
    Command::new("rgl")
        .bin_name("rgl")
        .about("Fast and minimal implementation of Regolith.")
        .author("ink0rr")
        .version(crate_version!())
        .arg(
            Arg::new("debug")
                .long("debug")
                .help("Print debug messages")
                .global(true)
                .action(ArgAction::SetTrue),
        )
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand(
            Command::new("add")
                .about("Add filter(s) to current project")
                .arg(
                    Arg::new("filters")
                        .num_args(1..)
                        .action(ArgAction::Set)
                        .required(true),
                )
                .arg(
                    Arg::new("force")
                        .short('f')
                        .long("force")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("clean").about("Clean the current project's cache and build files"),
        )
        .subcommand(
            Command::new("get")
                .about("Get all filters defined in current project")
                .arg(
                    Arg::new("force")
                        .short('f')
                        .long("force")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("init")
                .about("Initialize a new project in the current directory")
                .arg(
                    Arg::new("force")
                        .short('f')
                        .long("force")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("install")
                .alias("i")
                .about("Install tool(s)")
                .arg(
                    Arg::new("tools")
                        .num_args(1..)
                        .action(ArgAction::Set)
                        .required(true),
                )
                .arg(
                    Arg::new("force")
                        .short('f')
                        .long("force")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("list")
                .alias("ls")
                .about("List project's filters"),
        )
        .subcommand(
            Command::new("update")
                .aliases(["up", "upgrade"])
                .about("Checks for update and installs it if available")
                .arg(
                    Arg::new("force")
                        .short('f')
                        .long("force")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("remove")
                .alias("rm")
                .about("Remove filter(s) from current project")
                .arg(
                    Arg::new("filters")
                        .num_args(1..)
                        .action(ArgAction::Set)
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("run")
                .about("Runs Regolith with specified profile")
                .arg(Arg::new("profile").action(ArgAction::Set))
                .arg(
                    Arg::new("cached")
                        .long("cached")
                        .help("Use previous run output as cache")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("tool")
                .about("Runs a tool in the current project")
                .arg(Arg::new("tool_name").action(ArgAction::Set).required(true)),
        )
        .subcommand(
            Command::new("watch")
                .about("Watch for file changes and restart automatically")
                .arg(Arg::new("profile").action(ArgAction::Set))
                .arg(
                    Arg::new("no-cache")
                        .long("no-cache")
                        .help("Do not use previous run output as cache")
                        .action(ArgAction::SetTrue),
                ),
        )
}

fn run_command(matches: ArgMatches) -> Result<()> {
    let handle = match matches.subcommand_name() {
        // Don't trigger update check when running these commands
        Some("update" | "watch") => None,
        _ => Some(thread::spawn(rgl::check_for_update)),
    };
    match matches.subcommand() {
        Some(("add", matches)) => {
            let filters = matches
                .get_many::<String>("filters")
                .map(|filters| filters.collect())
                .unwrap();
            let force = matches.get_flag("force");
            commands::add_filters(filters, force).context("Error adding filter(s)")?;
        }
        Some(("clean", _)) => {
            commands::clean().context("Error cleaning files")?;
        }
        Some(("get", matches)) => {
            let force = matches.get_flag("force");
            commands::get_filters(force).context("Error getting filters")?;
        }
        Some(("init", matches)) => {
            let force = matches.get_flag("force");
            commands::init(force).context("Error initializing project")?;
        }
        Some(("install", matches)) => {
            let tools = matches
                .get_many::<String>("tools")
                .map(|tools| tools.collect())
                .unwrap();
            let force = matches.get_flag("force");
            commands::install_tools(tools, force).context("Error installing tool")?;
        }
        Some(("list", _)) => {
            commands::list().context("Error listing installed filters")?;
        }
        Some(("update", matches)) => {
            let force = matches.get_flag("force");
            commands::update(force).context("Error updating rgl")?;
        }
        Some(("remove", matches)) => {
            let filters = matches
                .get_many::<String>("filters")
                .map(|filters| filters.collect())
                .unwrap();
            commands::remove_filters(filters).context("Error removing filter(s)")?;
        }
        Some(("run", matches)) => {
            let profile = match matches.get_one::<String>("profile") {
                Some(profile) => profile,
                None => "default",
            };
            let cached = matches.get_flag("cached");
            commands::run_or_watch(profile, false, cached)
                .context(format!("Error running <b>{profile}</> profile"))?;
        }
        Some(("tool", matches)) => {
            let tool_name = matches.get_one::<String>("tool_name").unwrap();
            let args = env::args().skip(3).collect();
            commands::tool(tool_name, args)
                .context(format!("Error running tool <b>{tool_name}</>"))?;
        }
        Some(("watch", matches)) => {
            let profile = match matches.get_one::<String>("profile") {
                Some(profile) => profile,
                None => "default",
            };
            let no_cache = matches.get_flag("no-cache");
            commands::run_or_watch(profile, true, !no_cache)
                .context(format!("Error running <b>{profile}</> profile"))?;
        }
        _ => unreachable!(),
    }
    if let Some(handle) = handle {
        match handle.join().unwrap() {
            Ok(version) => {
                if let Some(version) = version {
                    rgl::prompt_update(version)?
                }
            }
            Err(e) => {
                warn!("Update check failed");
                e.chain().for_each(|e| log!("<yellow>[?]</> {e}"));
            }
        }
    }
    Ok(())
}
