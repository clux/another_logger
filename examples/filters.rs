//! An example using the Builder pattern API to configure the logger at run-time with module
//! filters. 
//!
//! The default output is `module::path: message`, and the "tag", which is the text to the left of
//! the colon, is colorized. This example allows the user to dynamically change the output based
//! on command line arguments.
//!
//! The [clap](https://crates.io/crates/clap) argument parser is used in this example, but loggerv
//! works with any argument parser.

extern crate ansi_term;
#[macro_use] extern crate log;
extern crate loggerv;
extern crate clap;

use clap::{Arg, App};

fn main() {
    // Add the following line near the beginning of the main function for an application to enable
    // colorized output on Windows 10. 
    //
    // Based on documentation for the ansi_term crate, Windows 10 supports ANSI escape characters,
    // but it must be enabled first using the `ansi_term::enable_ansi_support()` function. It is
    // conditionally compiled and only exists for Windows builds. To avoid build errors on
    // non-windows platforms, a cfg guard should be put in place.
    #[cfg(windows)] ansi_term::enable_ansi_support().unwrap();

    let args = App::new("app")
       .arg(Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity"))
       .arg(Arg::with_name("debug")
            .short("d")
            .long("debug")
            .help("Adds the line numbers to log statements"))
       .arg(Arg::with_name("no-module-path")
            .long("no-module-path")
            .help("Disables the module path in the log statements"))
       .arg(Arg::with_name("no-color")
            .long("no-color")
            .help("Disables colorized output"))
       .arg(Arg::with_name("level")
            .short("l")
            .long("level")
            .help("Adds the log level to the log statements. This will also surround the module path in square brackets."))
       .arg(Arg::with_name("filter")
            .short("f")
            .long("filter")
            .help("A comma-separated list of key-value pairs delimited with the equal sign, '=', where the key is a path to a module and the value is the level (error, warn, info, debug, or trace) of logging for the module/ley.")
            .takes_value(true))
       .get_matches();

    if let Some(f) = args.value_of("filter") {
        loggerv::Logger::new().filter(f)
    } else {
        loggerv::Logger::new()
    }.verbosity(args.occurrences_of("v"))
     .level(args.is_present("level"))
     .line_numbers(args.is_present("debug"))
     .module_path(!args.is_present("no-module-path"))
     .colors(!args.is_present("no-color"))
     .init()
     .unwrap();

    error!("This is always printed to stderr");
    warn!("This too is always printed to stderr");
    info!("This is optionally printed to stdout based on the verbosity");
    debug!("This is optionally printed to stdout based on the verbosity");
    trace!("This is optionally printed to stdout based on the verbosity");
}

