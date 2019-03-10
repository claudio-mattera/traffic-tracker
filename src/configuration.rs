use std::fs::File;
use std::io::prelude::*;

use env_logger;

use serde::Deserialize;

use clap::{App, Arg, ArgMatches};

use super::TrafficError;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub base_url: String,
    pub username: String,
    pub password: String,
}

pub fn load_configuration() -> Result<Configuration, TrafficError> {
    let matches = parse_command_line();

    setup_logging(matches.occurrences_of("v"));

    let mut file = File::open(matches.value_of("config").unwrap())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let configuration: Configuration = toml::from_str(&contents)?;
    Ok(configuration)
}

fn parse_command_line() -> ArgMatches<'static> {
    App::new("Traffic Tracker")
        .version("0.1.0")
        .author("Claudio Mattera <claudio@mattera.it>")
        .about("Track traffic in router Huawei E5172As-22")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .required(true)
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Verbosity level"),
        )
        .get_matches()
}

pub fn setup_logging(verbosity: u64) {
    let default_log_filter = match verbosity {
        0 => "traffic_tracker=warn",
        1 => "traffic_tracker=info",
        2 | _ => "traffic_tracker=debug",
    };
    let filter = env_logger::Env::default().default_filter_or(default_log_filter);
    env_logger::Builder::from_env(filter).init();
}

