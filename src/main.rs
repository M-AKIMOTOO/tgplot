mod cli;
mod data;
mod detail;
mod gnuplot;
mod model;

#[cfg(test)]
mod tests;

use std::env;
use std::process;

use cli::{HELP, parse_args};
use data::load_series;
use detail::print_detail;
use gnuplot::run_gnuplot;
use model::CliAction;

const BANNER: &str = "\
 _____ ____  ____  _     ___ _____
|_   _/ ___||  _ \\| |   / _ \\_   _|
  | || |  _ | |_) | |  | | | || |
  | || |_| ||  __/| |__| |_| || |
  |_| \\____||_|   |_____\\___/ |_|
";

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match parse_args(env::args().skip(1))? {
        CliAction::Help => {
            print!("{BANNER}\n");
            print!("{HELP}");
            Ok(())
        }
        CliAction::Detail => {
            print_detail();
            Ok(())
        }
        CliAction::Plot(config) => {
            let series = load_series(&config)?;
            if series.is_empty() {
                return Err("no plottable series were found".to_string());
            }
            run_gnuplot(&config, &series)
        }
    }
}
