extern crate log;
extern crate simplelog;

mod rgl;

use clap::{crate_version, Arg, ArgAction, Command};
use log::LevelFilter;
use simplelog::{error, ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

fn main() {
    let config = ConfigBuilder::new()
        .set_time_level(LevelFilter::Debug)
        .build();

    TermLogger::init(
        LevelFilter::Info,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    let matches = Command::new("rgl")
        .bin_name("rgl")
        .about("Not Regolith")
        .author("ink0rr")
        .version(crate_version!())
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand(
            Command::new("init")
                .about("Initializes a new Regolith project in the current directory"),
        )
        .subcommand(
            Command::new("install")
                .alias("i")
                .about("Downloads and installs Regolith filters from the internet, and adds them to the \"filterDefinitions\" list of the project's \"config.json\" file.")
                .arg(Arg::new("filters").num_args(0..).action(ArgAction::Set))
                .arg(Arg::new("force").short('f').long("force").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("run")
                .about("Runs Regolith with specified profile")
                .arg(Arg::new("profile").action(ArgAction::Set)),
        )
        .subcommand(
            Command::new("watch")
                .about("Watches project files and automatically runs Regolith when they change")
                .arg(Arg::new("profile").action(ArgAction::Set)),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("init", _)) => {
            if let Err(e) = rgl::init() {
                error!("Error initializing project\n{e}");
                std::process::exit(1);
            }
        }
        Some(("install", matches)) => {
            let filters: Option<Vec<&String>> = match matches.get_many::<String>("filters") {
                Some(filters) => Some(filters.collect::<Vec<&String>>()),
                None => None,
            };
            let force = matches.get_flag("force");
            match filters {
                Some(filters) => {
                    if let Err(e) = rgl::install_add(filters, force) {
                        error!("Error installing filter\n{e}");
                        std::process::exit(1);
                    }
                }
                None => {
                    if let Err(e) = rgl::install_filters(force) {
                        error!("Error installing filters\n{e}");
                        std::process::exit(1);
                    }
                }
            };
        }
        Some(("run", matches)) => {
            let profile = match matches.get_one::<String>("profile") {
                Some(profile) => profile,
                None => "default",
            };
            if let Err(e) = rgl::run_or_watch(profile, false) {
                error!("Error running <b>{profile}</> profile\n{e}");
                std::process::exit(1);
            }
        }
        Some(("watch", matches)) => {
            let profile = match matches.get_one::<String>("profile") {
                Some(profile) => profile,
                None => "default",
            };
            if let Err(e) = rgl::run_or_watch(profile, true) {
                error!("Error running <b>{profile}</> profile\n{e}");
                std::process::exit(1);
            }
        }
        _ => unreachable!(),
    }
}
