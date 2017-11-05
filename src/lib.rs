//! A simple `io::stdout` and `io::stderr` writing `Logger` implementation from the
//! `log` crate, using the `ansi_term` crate for colors and configured at runtime,
//! or at compile time with simple function calls. Designed for simple CLIs.
//!
//! This library only comes with 3 public ways to initialize the global logger.
//! Ensure you call one of these exactly once early in your rust program as shown
//! in one of the examples below.
//!
//! ## Example
//! The standard example with `clap` as the arg parser.
//!
//! ```
//! #[macro_use] extern crate log;
//! extern crate clap;
//! extern crate loggerv;
//!
//! use clap::{Arg, App};
//!
//! fn main() {
//!     let args = App::new("app")
//!                    .arg(Arg::with_name("v")
//!                             .short("v")
//!                             .multiple(true)
//!                             .help("Sets the level of verbosity"))
//!                    .get_matches();
//!
//!     loggerv::init_with_verbosity(args.occurrences_of("v")).unwrap();
//!
//!     error!("this is always printed");
//!     warn!("this too, and it's printed to stderr");
//!     info!("this is optional");  // for ./app -v or higher
//!     debug!("this is optional"); // for ./app -vv or higher
//!     trace!("this is optional"); // for ./app -vvv
//! }
//! ```
//!
//! But obviously use whatever argument parsing methods you prefer.
//!
//! ## Example
//! For a compile time switch, all you really need is `log` (for the macros)
//! and `loggerv` for how to print what's being sent to the macros.
//!
//! ```
//! #[macro_use] extern crate log;
//! extern crate loggerv;
//!
//! use log::LogLevel;
//!
//! fn main() {
//!     loggerv::init_with_level(LogLevel::Info).unwrap();
//!     debug!("this is a debug {}", "message");
//!     error!("this is printed by default");
//! }
//! ```
//!
//! # Example
//! If you don't really care at all you could just use the plain `init_quiet` function
//! to only show warnings and errors:
//!
//! ```
//! #[macro_use] extern crate log;
//! extern crate loggerv;
//!
//! fn main() {
//!     loggerv::init_quiet().unwrap();
//!     info!("hidden");
//!     error!("this is printed by default");
//! }
//! ```
//!
//!
//! See the documentation for the log crate for more information about its API.
//!

#[cfg(test)]
#[macro_use]
extern crate log;

#[cfg(not(test))]
extern crate log;

extern crate atty;
extern crate ansi_term;

use log::{Log, LogLevel, LogMetadata, LogRecord, SetLoggerError};
use std::io::{self, Write};
use ansi_term::Colour;

struct VLogger {
    log_level: LogLevel,
    colors: bool,
}

impl VLogger {
    fn new(log_level: LogLevel) -> VLogger {
        let colors = atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stderr);
        VLogger { log_level, colors }
    }
}

fn level_style(l: LogLevel) -> Colour {
    match l {
        LogLevel::Error => Colour::Fixed(9), // bright red
        LogLevel::Warn => Colour::Fixed(11), // bright yellow
        LogLevel::Info => Colour::Fixed(10), // bright green
        LogLevel::Debug => Colour::Fixed(7), // light grey
        LogLevel::Trace => Colour::Fixed(8), // grey
    }
}

impl Log for VLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.log_level
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            let level = if self.colors {
                level_style(record.level())
                    .paint(record.location().module_path())
                    .to_string()
            } else {
                record.location().module_path().to_string()
            };

            let _ = writeln!(&mut io::stderr(), "{}: {}", level, record.args());
        }
    }
}

/// Initialize loggerv with a maximal log level.
///
/// See the main loggerv documentation page for an example.
pub fn init_with_level(log_level: LogLevel) -> Result<(), SetLoggerError> {
    log::set_logger(|max_log_level| {
        max_log_level.set(log_level.to_log_level_filter());
        Box::new(VLogger::new(log_level))
    })
}

/// Initialize loggerv with a verbosity level.
///
/// Intended to be used with an arg parser counting the amount of -v flags.
/// See the main loggerv documentation page for an example.
pub fn init_with_verbosity(verbosity: u64) -> Result<(), SetLoggerError> {
    init_with_level(match verbosity {
        0 => LogLevel::Warn,  // default
        1 => LogLevel::Info,  // -v
        2 => LogLevel::Debug, // -vv
        _ => LogLevel::Trace, // -vvv and above
    })
}

/// Initializes loggerv with only warnings and errors.
///
/// See the main loggerv documentation page for an example.
pub fn init_quiet() -> Result<(), SetLoggerError> {
    init_with_level(LogLevel::Warn)
}

#[cfg(test)]
mod tests {
    use init_with_verbosity;

    #[test]
    fn init_and_macros() {
        let l = init_with_verbosity(3);
        assert_eq!(l.is_ok(), true);
        error!("error log");
        warn!("warn log");
        info!("info log");
        debug!("debug log");
        trace!("trace log");
    }
}
