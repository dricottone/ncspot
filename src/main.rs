#[macro_use]
extern crate cursive;
#[macro_use]
extern crate serde;

use std::path::PathBuf;

use application::{setup_logging, Application};
use ncspot::program_arguments;

mod application;
mod authentication;
mod cli;
mod command;
mod commands;
mod config;
mod events;
mod ext_traits;
mod library;
mod model;
mod panic;
mod queue;
mod spotify;
mod spotify_api;
mod spotify_url;
mod spotify_worker;
mod traits;
mod ui;
mod utils;

fn main() {
    // Set a custom backtrace hook that writes the backtrace to a file instead of stdout, since
    // stdout is most likely in use by Cursive.
    panic::register_backtrace_panic_handler();

    // Parse the command line arguments.
    let matches = program_arguments().get_matches();

    // Enable debug logging to a file if specified on the command line.
    if let Some(filename) = matches.get_one::<PathBuf>("debug") {
        setup_logging(filename).expect("logger could not be initialized");
    }

    match matches.subcommand() {
        Some(("info", _subcommand_matches)) => cli::info(),
        Some((_, _)) => unreachable!(),
        None => {
            // Create the application.
            let mut application = Application::new().unwrap();

            // Start the application event loop.
            application.run()
        }
    }
}
