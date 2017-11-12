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

pub const DEFAULT_COLORS: bool = true;
pub const DEFAULT_SEPARATOR: &str = ": ";
pub const DEFAULT_LEVEL: LogLevel = LogLevel::Warn;
pub const DEFAULT_LINE_NUMBERS: bool = false;
pub const DEFAULT_MODULE_PATH: bool = true;

fn level_style(l: LogLevel) -> Colour {
    match l {
        LogLevel::Error => Colour::Fixed(9), // bright red
        LogLevel::Warn => Colour::Fixed(11), // bright yellow
        LogLevel::Info => Colour::Fixed(10), // bright green
        LogLevel::Debug => Colour::Fixed(7), // light grey
        LogLevel::Trace => Colour::Fixed(8), // grey
    }
}

pub struct Logger {
    colors: bool,
    line_numbers: bool,
    level: LogLevel,
    module_path: bool,
    separator: String, 
}


impl Logger {
    /// Creates a new instance of the verbosity-based logger.
    ///
    /// The default level is WARN. Color is enabled if the parent application or library is running
    /// from a terminal, i.e. running a tty. The default separator is the ": " string. The default
    /// output format is `LEVEL [module path]: message`.
    pub fn new() -> Logger {
        Logger { 
            colors: DEFAULT_COLORS && atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stderr),
            line_numbers: DEFAULT_LINE_NUMBERS,
            level: DEFAULT_LEVEL, 
            module_path: DEFAULT_MODULE_PATH,
            separator: String::from(DEFAULT_SEPARATOR),
        }
    }

    /// Sets the separator string.
    ///
    /// The separator is the string between the "tag" and the message that make up a log statement.
    /// The tag will be colorized if enabled, while the message will not. The default is `: `.
    pub fn separator(mut self, s: &str) -> Self {
        self.separator = String::from(s);
        self
    }

    /// Enables or disables colorizing the output. 
    ///
    /// If the logger is _not_ used in a terminal, then
    /// the output is _not_ colorized regardless of the parameter value.
    pub fn colors(mut self, c: bool) -> Self {
        self.colors = c && atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stderr);
        self
    }

    /// Enables or disables including line numbers in the "tag" portion of the log statement. The
    /// tag is the text to the left of the separator.
    pub fn line_numbers(mut self, l: bool) -> Self {
        self.line_numbers = l;
        self
    }

    /// Explicitly sets the log level instead through a verbosity.
    pub fn level(mut self, l: LogLevel) -> Self {
        self.level = l;
        self
    }

    /// Enables or disables including the module path in the "tag" portion of the log statement.
    /// The tag is the text to the left of the separator.
    pub fn module_path(mut self, m: bool) -> Self {
        self.module_path = m;
        self
    }

    /// Converts the verbosity to a log level.
    ///
    /// A verbosity of zero (0) is the default, which means ERROR and WARN log statements are
    /// printed to `stderr`. No other log statements are printed on any of the standard streams
    /// (`stdout` or `stderr`). As the verbosity is increased, the log level is increased and more
    /// log statements will be printed to `stdout`. A verbosity of 1 will print INFO log statements
    /// to `stdout` in addition to ERROR and WARN. A verbosity of 2 will print INFO and DEBUG log
    /// statements to `stdout`. A verbosity of 3 or higher will print INFO, DEBUG, and TRACE
    /// log statements to `stdout` with ERROR and WARN statements printed to `stderr`.
    pub fn verbosity(mut self, v: u64) -> Self {
        self.level = match v {
                0 => LogLevel::Warn,  // default
                1 => LogLevel::Info,  // -v
                2 => LogLevel::Debug, // -vv
                _ => LogLevel::Trace, // -vvv and above
        };
        self
    }

    /// Initializes the logger. 
    ///
    /// This also consumes the logger and cannot no longer be modified after initialization.
    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_logger(|max_level| {
            max_level.set(self.level.to_log_level_filter());
            Box::new(self)
        })
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            let level = record.level();
            let module_path = if self.module_path {
                format!(" [{}]", record.location().module_path())
            } else {
                String::new()
            };
            let line = if self.line_numbers {
                format!(" (line {})", record.location().line())
            } else {
                String::new()
            };
            let tag = if self.colors {
                level_style(level)
                    .paint(format!("{}{}{}", level, module_path, line))
                    .to_string()
            } else {
                format!("{}{}{}", level, module_path, line)
            };
            if level <= LogLevel::Warn {
                let _ = writeln!(&mut io::stderr(), "{}{}{}", tag, self.separator, record.args());
            } else {
                println!("{}{}{}", tag, self.separator, record.args());
            }
        }
    }
}

impl Default for Logger {
    fn default() -> Logger {
        Logger::new()
    }
}

/// Initialize loggerv with a maximal log level.
///
/// See the main loggerv documentation page for an example.
pub fn init_with_level(level: LogLevel) -> Result<(), SetLoggerError> {
    Logger::new().level(level).init()
}

/// Initialize loggerv with a verbosity level.
///
/// Intended to be used with an arg parser counting the amount of -v flags.
/// See the main loggerv documentation page for an example.
pub fn init_with_verbosity(verbosity: u64) -> Result<(), SetLoggerError> {
    Logger::new().verbosity(verbosity).init()
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
