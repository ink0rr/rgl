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
        Some(("run", run_matches)) => {
            let profile = match run_matches.get_one::<String>("profile") {
                Some(profile) => profile,
                None => "default",
            };
            if let Err(e) = rgl::run_or_watch(profile, false) {
                error!("Error running <b>{profile}</> profile\n{e}");
                std::process::exit(1);
            }
        }
        Some(("watch", watch_matches)) => {
            let profile = match watch_matches.get_one::<String>("profile") {
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
