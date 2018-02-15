//! A simple `io::stdout` and `io::stderr` writing `Logger` implementation from the
//! `log` crate, using the `ansi_term` crate for colors and configured at runtime via a verbosity
//! or at compile time with simple function calls. Designed for simple Command Line Interfaces
//! (CLIs).
//!
//! This library includes a Builder pattern API for configuring a logger and three initializing
//! helper functions to create a default logger. Ensure you create and initialize only once
//! a global logger with the Builder pattern API or use one of the three public helper functions
//! early in your program as shown in the examples below.
//!
//! The default configuration colorizes the "tag" portion of the log statement, where the tag is
//! the text to the left of a separator, defaulted as the colon (`:`). The message is the
//! portion to the right of the separator and it is _not_ ever colorized. The tag includes only the
//! module path and the separator by default.
//!
//! ## Example
//!
//! The standard example with [clap](https://crates.io/crates/clap) as the arg parser using the
//! default configuration.
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
//!         .arg(Arg::with_name("v")
//!             .short("v")
//!             .multiple(true)
//!             .help("Sets the level of verbosity"))
//!         .get_matches();
//!
//!     loggerv::init_with_verbosity(args.occurrences_of("v")).unwrap();
//!
//!     error!("This is always printed");
//!     warn!("This too is always printed to stderr");
//!     info!("This is optionally printed to stdout");  // for ./app -v or higher
//!     debug!("This is optionally printed to stdout"); // for ./app -vv or higher
//!     trace!("This is optionally printed to stdout"); // for ./app -vvv
//! }
//! ```
//!
//! But obviously use whatever argument parsing methods you prefer.
//!
//! ## Example
//!
//! For a compile time switch, all you really need is `log` (for the macros) and `loggerv` for how
//! to print what's being sent to the macros with the default configuration.
//!
//! ```
//! #[macro_use] extern crate log;
//! extern crate loggerv;
//!
//! use log::Level;
//!
//! fn main() {
//!     loggerv::init_with_level(Level::Info).unwrap();
//!     debug!("This is a debug {}", "message"); // Not printed to stdout
//!     error!("This is printed by default");    // Printed to stderr
//! }
//! ```
//!
//! ## Example
//!
//! If you don't really care at all you could just use the plain `init_quiet` function to only show
//! warnings and errors with the default configuration:
//!
//! ```
//! #[macro_use] extern crate log;
//! extern crate loggerv;
//!
//! fn main() {
//!     loggerv::init_quiet().unwrap();
//!     info!("Hidden");
//!     error!("This is printed by default");
//! }
//! ```
//!
//! ## Example
//!
//! If you want to configure the output, the Builder pattern API can be used.
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
//!     loggerv::Logger::new()
//!         .verbosity(args.occurrences_of("v"))
//!         .level(true)
//!         .line_numbers(true)
//!         .separator(" = ")
//!         .module_path(false)
//!         .colors(false)
//!         .init()
//!         .unwrap();
//!
//!     error!("This is always printed to stderr");
//!     warn!("This too is always printed to stderr");
//!     info!("This is optionally printed to stdout");  // for ./app -v or higher
//!     debug!("This is optionally printed to stdout"); // for ./app -vv or higher
//!     trace!("This is optionally printed to stdout"); // for ./app -vvv
//! }
//! ```
//!
//! See the [documentation](https://docs.rs/log/0.4.1/log/) for the
//! [log](https://crates.io/crates/log) crate for more information about its API.
//!

extern crate atty;
extern crate ansi_term;
extern crate env_logger;
extern crate log;

use env_logger::filter::{Builder, Filter};
use log::SetLoggerError;
use std::io::{self, Write};
use ansi_term::Colour;

pub const DEFAULT_COLORS: bool = true;
pub const DEFAULT_DEBUG_COLOR: Colour = Colour::Fixed(7); // light grey
pub const DEFAULT_ERROR_COLOR: Colour = Colour::Fixed(9); // bright red
pub const DEFAULT_INCLUDE_LEVEL: bool = false;
pub const DEFAULT_INCLUDE_LINE_NUMBERS: bool = false;
pub const DEFAULT_INCLUDE_MODULE_PATH: bool = true;
pub const DEFAULT_INFO_COLOR: Colour = Colour::Fixed(10); // bright green
pub const DEFAULT_OFFSET: u64 = 1;
pub const DEFAULT_SEPARATOR: &str = ": ";
pub const DEFAULT_TRACE_COLOR: Colour = Colour::Fixed(8); // grey
pub const DEFAULT_WARN_COLOR: Colour = Colour::Fixed(11); // bright yellow

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Output {
    Stderr,
    Stdout,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Level {
    output: Output,
    color: Colour,
}

struct InnerLogger {
    filter: Filter,
    write: Box<Fn(&mut Write, &log::Record) -> io::Result<()> + Sync + Send>,
    select_output: Box<Fn(&log::Level) -> Output + Sync + Send>,
}

impl InnerLogger {
    fn filter(&self) -> log::LevelFilter {
        self.filter.filter()
    }
}

impl log::Log for InnerLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.filter.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        if self.filter.matches(record) {
            match (self.select_output)(&record.level()) {
                Output::Stderr => (self.write)(&mut io::stderr(), &record).expect("Write to stderr"),
                Output::Stdout => (self.write)(&mut io::stdout(), &record).expect("Write to stdout"),
            };
        }
    }

    fn flush(&self) {}
}

#[derive(Debug)]
pub struct Logger {
    colors: bool,
    builder: Builder,
    include_level: bool,
    include_line_numbers: bool,
    include_module_path: bool,
    separator: String,
    verbosity: Option<u64>,
    error: Level,
    warn: Level,
    info: Level,
    debug: Level,
    trace: Level,
}

impl Logger {
    /// Creates a new instance of the verbosity-based logger.
    ///
    /// The default level is WARN. Color is enabled if the parent application or library is running
    /// from a terminal, i.e. running a tty. The default separator is the ": " string. The default
    /// output format is `module path: message`. The following default colors are used:
    ///
    /// | Level | Color         |
    /// |-------|---------------|
    /// | Error | Bright Red    |
    /// | Warn  | Bright Yellow |
    /// | Info  | Bright Green  |
    /// | Debug | Light Grey    |
    /// | Trace | Grey          |
    pub fn new() -> Logger {
        Logger {
            builder: Builder::from_env("RUST_LOG"),
            colors: DEFAULT_COLORS && atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stderr),
            include_level: DEFAULT_INCLUDE_LEVEL,
            include_line_numbers: DEFAULT_INCLUDE_LINE_NUMBERS,
            include_module_path: DEFAULT_INCLUDE_MODULE_PATH,
            separator: String::from(DEFAULT_SEPARATOR),
            verbosity: None,
            error: Level {
                output: Output::Stderr,
                color: DEFAULT_ERROR_COLOR,
            },
            warn: Level {
                output: Output::Stderr,
                color: DEFAULT_WARN_COLOR,
            },
            info: Level {
                output: Output::Stdout,
                color: DEFAULT_INFO_COLOR,
            },
            debug: Level {
                output: Output::Stdout,
                color: DEFAULT_DEBUG_COLOR,
            },
            trace: Level {
                output: Output::Stdout,
                color: DEFAULT_TRACE_COLOR,
            }
        }
    }

    /// Sets the color for a level.
    ///
    /// # Example
    ///
    /// ```
    /// #[macro_use] extern crate log;
    /// extern crate loggerv;
    /// extern crate ansi_term;
    ///
    /// use log::Level;
    /// use ansi_term::Colour;
    ///
    /// fn main() {
    ///     loggerv::Logger::new()
    ///         .color(&Level::Error, Colour::Fixed(7))
    ///         .init()
    ///         .unwrap();
    ///
    ///     error!("This is printed in light grey instead of bright red");
    /// }
    /// ```
    pub fn color(mut self, l: &log::Level, c: Colour) -> Self {
        match *l {
            log::Level::Error => self.error.color = c,
            log::Level::Warn => self.warn.color = c,
            log::Level::Info => self.info.color = c,
            log::Level::Debug => self.debug.color = c,
            log::Level::Trace => self.trace.color = c,
        }
        self
    }

    /// Sets the separator string.
    ///
    /// The separator is the string between the "tag" and the message that make up a log statement.
    /// The tag will be colorized if enabled, while the message will not. The default is `: `.
    ///
    /// If the level, line numbers, and module path are all _not_ included in the log statement,
    /// then the separator is changed to the empty string to avoid printing a lone string or
    /// character before each message portion of the log statement.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate log;
    /// extern crate loggerv;
    ///
    /// fn main() {
    ///     loggerv::Logger::new()
    ///         .separator(" = ")
    ///         .init()
    ///         .unwrap();
    ///
    ///     error!("This is printed with an equal sign between the module path and this message");
    /// }
    /// ```
    pub fn separator(mut self, s: &str) -> Self {
        self.separator = String::from(s);
        self
    }

    /// Enables or disables colorizing the output.
    ///
    /// If the logger is _not_ used in a terminal, then the output is _not_ colorized regardless of
    /// this value.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate log;
    /// extern crate loggerv;
    ///
    /// fn main() {
    ///     loggerv::Logger::new()
    ///         .colors(false)
    ///         .init()
    ///         .unwrap();
    ///
    ///     error!("This is printed without any colorization");
    /// }
    /// ```
    pub fn colors(mut self, c: bool) -> Self {
        self.colors = c && atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stderr);
        self
    }

    /// Disables colorizing the output.
    ///
    /// The default is to colorize the output unless `stdout` and `stderr` are redirected or piped,
    /// i.e. not a tty.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate log;
    /// extern crate loggerv;
    ///
    /// fn main() {
    ///     loggerv::Logger::new()
    ///         .no_colors()
    ///         .init()
    ///         .unwrap();
    ///
    ///     error!("This is printed without any colorization");
    /// }
    /// ```
    pub fn no_colors(mut self) -> Self {
        self. colors = false;
        self
    }

    /// Enables or disables including line numbers in the "tag" portion of the log statement.
    ///
    /// The tag is the text to the left of the separator.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate log;
    /// extern crate loggerv;
    ///
    /// fn main() {
    ///     loggerv::Logger::new()
    ///         .line_numbers(true)
    ///         .init()
    ///         .unwrap();
    ///
    ///     error!("This is printed with the module path and the line number surrounded by
    ///     parentheses");
    /// }
    /// ```
    pub fn line_numbers(mut self, i: bool) -> Self {
        self.include_line_numbers = i;
        self
    }

    /// Enables or disables including the level in the log statement's tag portion. The tag of the
    /// log statement is the text to the left of the separator.
    ///
    /// If the level and the module path are both inculded, then the module path is surrounded by
    /// square brackets.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate log;
    /// extern crate loggerv;
    ///
    /// fn main() {
    ///     loggerv::Logger::new()
    ///         .level(true)
    ///         .init()
    ///         .unwrap();
    ///
    ///     error!("This is printed with the 'ERROR' and the module path is surrounded in square
    ///     brackets");
    /// }
    /// ```
    pub fn level(mut self, i: bool) -> Self {
        self.include_level = i;
        self
    }

    /// Explicitly sets the log level instead of through a verbosity.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate log;
    /// extern crate loggerv;
    ///
    /// fn main() {
    ///     loggerv::Logger::new()
    ///         .max_level(log::Level::Info)
    ///         .init()
    ///         .unwrap();
    ///
    ///     error!("This is printed to stderr");
    ///     warn!("This is printed to stderr");
    ///     info!("This is printed to stdout");
    ///     debug!("This is not printed to stdout");
    ///     trace!("This is not printed to stdout");
    /// }
    /// ```
    pub fn max_level(mut self, l: log::Level) -> Self {
        self.builder.filter(None, l.to_level_filter());
        // It is important to set the Verbosity to None here because later with the `init` method,
        // a `None` value indicates the verbosity has _not_ been set or overriden by using this
        // method (`max_level`). If the verbosity is some value, then it will be used and the use
        // of this method will be dismissed.
        self.verbosity = None;
        self
    }

    /// Enables or disables including the module path in the "tag" portion of the log statement.
    ///
    /// The tag is the text to the left of the separator. The default is to include the module
    /// path. Ifthe level is also included, the module path is surrounded by square brackets.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate log;
    /// extern crate loggerv;
    ///
    /// fn main() {
    ///     loggerv::Logger::new()
    ///         .module_path(false)
    ///         .init()
    ///         .unwrap();
    ///
    ///     error!("This is printed without leading module path and separator");
    /// }
    /// ```
    pub fn module_path(mut self, i: bool) -> Self {
        self.include_module_path = i;
        self
    }

    /// Disables the module path in the "tag" portion of the log statement.
    ///
    /// The tag is the text to the left of the separator. The default is to include the module
    /// path.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate log;
    /// extern crate loggerv;
    ///
    /// fn main() {
    ///     loggerv::Logger::new()
    ///         .no_module_path()
    ///         .init()
    ///         .unwrap();
    ///
    ///     error!("This is printed without leading module path and separator");
    /// }
    /// ```
    pub fn no_module_path(mut self) -> Self {
        self.include_module_path = false;
        self
    }

    /// Sets the output for a level.
    ///
    /// The output is either `stderr` or `stdout`. The default is for ERROR and WARN to be written
    /// to `stderr` and INFO, DEBUG, and TRACE to `stdout`.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate log;
    /// extern crate loggerv;
    ///
    /// use log::Level;
    /// use loggerv::Output;
    ///
    /// fn main() {
    ///     loggerv::Logger::new()
    ///         .output(&Level::Error, Output::Stdout)
    ///         .output(&Level::Warn, Output::Stdout)
    ///         .output(&Level::Info, Output::Stderr)
    ///         .output(&Level::Debug, Output::Stderr)
    ///         .output(&Level::Trace, Output::Stderr)
    ///         .verbosity(0)
    ///         .init()
    ///         .unwrap();
    ///
    ///     error!("This is printed on stdout instead of stderr");
    ///     warn!("This is printed on stdout instead of stderr");
    ///     info!("This is printed on stderr instead of stdout");
    ///     debug!("This is printed on stderr instead of stdout");
    ///     trace!("This is printed on stderr instead of stdout");
    /// }
    /// ```
    pub fn output(mut self, l: &log::Level, o: Output) -> Self {
        match *l {
            log::Level::Error => self.error.output = o,
            log::Level::Warn => self.warn.output = o,
            log::Level::Info => self.info.output = o,
            log::Level::Debug => self.debug.output = o,
            log::Level::Trace => self.trace.output = o,
        }
        self
    }

    /// Sets the level based on verbosity and the base level inherited from the environment.
    ///
    /// A verbosity of zero (0) indicates the default level is inherited from the environment via
    /// the `RUST_LOG` environment variable. As the verbosity is increased, the log level is increased and more
    /// log statements will be printed to `stdout`.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate log;
    /// extern crate loggerv;
    ///
    /// fn main() {
    ///     loggerv::Logger::new()
    ///         .verbosity(1)
    ///         .init()
    ///         .unwrap();
    ///
    ///     error!("This is printed to stderr");
    ///     warn!("This is printed to stderr");
    ///     info!("This is printed to stdout");
    ///     debug!("This is not printed to stdout");
    ///     trace!("This is not printed to stdout");
    /// }
    /// ```
    pub fn verbosity(mut self, v: u64) -> Self {
        self.verbosity = Some(v);
        self
    }

    /// Adds filters.
    ///
    /// The pattern and format is identical to the
    /// [env_logger](https://docs.rs/env_logger/0.5.3/env_logger) filter syntax and implementation.
    /// This takes a comma-separated list of logging directives. Directives are in the form:
    ///
    /// ```text
    /// path::to::module=level
    /// ```
    ///
    /// # Example
    ///
    /// The following example will print logging statements from the `hello` module at the ERROR,
    /// WARN, and INFO levels, while logging statements from the `goodbye` module will be printed
    /// at the ERROR level only. The `filter` method can be used for individual directives or for
    /// a comma-separated list, where ERROR and WARN logging statements will be printed from the
    /// `welcome` module, TRACE, DEBUG, INFO, WARN, and ERROR logging statements will be printed
    /// from the `thank::you` module, and ERROR, WARN, INFO, and DEBUG statements will be printed
    /// from the `bye` module.
    ///
    /// ```rust
    /// #[macro_use] extern crate log;
    /// extern crate loggerv;
    ///
    /// fn main() {
    ///     loggerv::Logger::new()
    ///         .filter("hello=info")
    ///         .filter("goodbye=error")
    ///         .filter("welcome=warn,thank::you=trace,bye=debug")
    ///         .init()
    ///         .unwrap();
    ///     
    ///     error!("This is printed to stderr");
    ///     warn!("This is printed to stderr");
    ///     info!("This is printed to stdout");
    ///     debug!("This is not printed to stdout");
    ///     trace!("This is not printed to stdout");
    /// }
    /// ```
    pub fn filter(mut self, directives: &str) -> Self {
        self.builder.parse(directives);
        self
    }

    /// Initializes the logger.
    ///
    /// This also consumes the logger. It cannot be further modified after initialization.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[macro_use] extern crate log;
    /// extern crate loggerv;
    ///
    /// fn main() {
    ///     loggerv::Logger::new()
    ///         .init()
    ///         .unwrap();
    ///
    ///     error!("This is printed to stderr");
    ///     warn!("This is printed to stderr");
    ///     info!("This is not printed to stdout");
    ///     debug!("This is not printed to stdout");
    ///     trace!("This is not printed to stdout");
    /// }
    /// ```
    ///
    /// # Example
    ///
    /// If the tag will be empty because the level, line numbers, and module path were all
    /// disabled, then the separator is changed to the empty string to avoid writing a long
    /// character in front of each message for each log statement.
    ///
    ///
    /// ```rust
    /// #[macro_use] extern crate log;
    /// extern crate loggerv;
    ///
    /// fn main() {
    ///     loggerv::Logger::new()
    ///         .module_path(false)
    ///         .level(false)
    ///         .line_numbers(false)
    ///         .init()
    ///         .unwrap();
    ///
    ///     error!("This is printed to stderr without the separator");
    ///     warn!("This is printed to stderr without the separator");
    ///     info!("This is not printed to stdout");
    ///     debug!("This is not printed to stdout");
    ///     trace!("This is not printed to stdout");
    /// }
    /// ```
    pub fn init(mut self) -> Result<(), SetLoggerError> {
        // If there is no level, line number, or module path in the tag, then the tag will always
        // be empty. The separator should also be empty so only the message component is printed
        // for the log statement; otherwise, there is a weird floating colon in front of every log
        // statement.
        //
        // It is better to do it here than in the `log` function because it only has to be
        // determined once at initialization as opposed to every call to the `log` function. So
        // a potentially slight performance improvement.
        if !self.include_level && !self.include_line_numbers && !self.include_module_path {
            self.separator = String::new();
        }
        // Build the filter now to get the maximum log level for calculation of the log level based
        // on verbosity. The offset is now determined from the environment instead of a 
        // `base_level` method. This is a temporary filter, just to get the "filter" based on the
        // environment. The builder will be reused to adjust the filter based on verbosity.
        // Luckily, the `build` method does not consume the builder.
        let filter = self.builder.build();
        let offset = match filter.filter() {
            log::LevelFilter::Off => DEFAULT_OFFSET,
            log::LevelFilter::Error => 0,
            log::LevelFilter::Warn => 1,
            log::LevelFilter::Info => 2,
            log::LevelFilter::Debug => 3,
            log::LevelFilter::Trace => 4,
        };
        // The level is set based on verbosity only if the `verbosity` method has been used and
        // _not_ overridden by a later call to the `max_level` method. If neither the `verbosity` or
        // `max_level` method is used, then the level set by the environment is used because it is
        // set within the `new` function when the `env_logger` filter Builder is created. It makes
        // more sense to calculate the level based on verbosity _after_ all configuration methods
        // have been called as opposed to during the call to the `verbosity` method. This change
        // enables the offset feature so that the `max_level` method can be used at any time during
        // the "building" procedure before the call to `init`. Otherwise, calling the `max_level`
        // _after_ the `verbosity` method would have no effect and be difficult to communicate this
        // limitation to users.
        if let Some(v) = self.verbosity {
            match v + offset {
                0 => self.builder.filter(None, log::Level::Error.to_level_filter()),
                1 => self.builder.filter(None, log::Level::Warn.to_level_filter()),
                2 => self.builder.filter(None, log::Level::Info.to_level_filter()),
                3 => self.builder.filter(None, log::Level::Debug.to_level_filter()),
                _ => self.builder.filter(None, log::Level::Trace.to_level_filter()),
            };
        }
        // Build the internal logger from the configurations. The values are "moved" into the
        // respective closures, so cloning is needed for values that do not implement the `Copy`
        // trait. This prevents having to duplicate all of these fields in the internal logger and
        // avoids a bunch of `Option` type fields. Basically, the `Logger` struct becomes
        // a builder, but to the outside world, the API and functionality is the same. See the
        // `env_logger` crate, where the builder pattern is heavily used and was the "inspiration"
        // for this implementation.
        let separator = self.separator.clone();
        let error_color = self.error.color.clone();
        let warn_color = self.warn.color.clone();
        let info_color = self.info.color.clone();
        let debug_color = self.debug.color.clone();
        let trace_color = self.trace.color.clone();
        let error_output = self.error.output.clone();
        let warn_output = self.warn.output.clone();
        let info_output = self.info.output.clone();
        let debug_output = self.debug.output.clone();
        let trace_output = self.trace.output.clone();
        let logger = InnerLogger {
            // We need to rebuild the filter after determining the level based on verbosity. If we
            // use the temporary `filter` variable from earlier to determine the base level, then
            // adjustments to the filter based on the verbosity will be lost. 
            filter: self.builder.build(),
            select_output: Box::new(move |level| {
                match *level {
                    log::Level::Error => error_output,
                    log::Level::Warn => warn_output,
                    log::Level::Info => info_output,
                    log::Level::Debug => debug_output,
                    log::Level::Trace => trace_output,
                }
            }),
            write: Box::new(move |buf, record| {
                let level = record.level();
                let level_text = if self.include_level {
                    level.to_string()
                } else {
                    String::new()
                };
                let module_path_text = if self.include_module_path {
                    let path = record.module_path().unwrap_or("unknown");
                    if self.include_level {
                        format!(" [{}]", path)
                    } else {
                        path.into()
                    }
                } else {
                    String::new()
                };
                let line_text = if self.include_line_numbers {
                    if let Some(l) = record.line() {
                        format!(" (line {})", l)
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                let mut tag = format!("{}{}{}", level_text, module_path_text, line_text);
                if self.colors {
                    let color = match level {
                        log::Level::Error => error_color,
                        log::Level::Warn => warn_color,
                        log::Level::Info => info_color,
                        log::Level::Debug => debug_color,
                        log::Level::Trace => trace_color,
                    };
                    tag = color.paint(tag).to_string();
                }
                writeln!(buf, "{}{}{}", tag, separator, record.args())
            }),
        };
        log::set_max_level(logger.filter());
        log::set_boxed_logger(Box::new(logger))
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
pub fn init_with_level(level: log::Level) -> Result<(), SetLoggerError> {
    Logger::new().max_level(level).init()
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
    init_with_level(log::Level::Warn)
}

#[cfg(test)]
mod tests {
    use log;
    use ansi_term::Colour;
    use super::*;

    #[test]
    fn defaults_are_correct() {
        let logger = Logger::new();
        assert_eq!(logger.include_level, DEFAULT_INCLUDE_LEVEL);
        assert_eq!(logger.include_line_numbers, DEFAULT_INCLUDE_LINE_NUMBERS);
        assert_eq!(logger.include_module_path, DEFAULT_INCLUDE_MODULE_PATH);
        assert_eq!(logger.colors, DEFAULT_COLORS);
        assert_eq!(logger.separator, String::from(DEFAULT_SEPARATOR));
        assert_eq!(logger.error.color, DEFAULT_ERROR_COLOR);
        assert_eq!(logger.warn.color, DEFAULT_WARN_COLOR);
        assert_eq!(logger.info.color, DEFAULT_INFO_COLOR);
        assert_eq!(logger.debug.color, DEFAULT_DEBUG_COLOR);
        assert_eq!(logger.trace.color, DEFAULT_TRACE_COLOR);
    }

    #[test]
    fn color_works() {
        let logger = Logger::new().color(&log::Level::Trace, Colour::Fixed(11));
        assert_eq!(logger.trace.color, Colour::Fixed(11));
    }

    #[test]
    fn separator_works() {
        const EXPECTED: &str = " = ";
        let logger = Logger::new().separator(EXPECTED);
        assert_eq!(logger.separator, EXPECTED);
    }

    #[test]
    fn colors_works() {
        let logger = Logger::new().colors(false);
        assert!(!logger.colors);
    }

    #[test]
    fn no_colors_works() {
        let logger = Logger::new().no_colors();
        assert!(!logger.colors);
    }

    #[test]
    fn line_numbers_works() {
        let logger = Logger::new().line_numbers(true);
        assert!(logger.include_line_numbers);
    }

    #[test]
    fn level_works() {
        let logger = Logger::new().level(true);
        assert!(logger.include_level);
    }

    #[test]
    fn max_level_works() {
        let logger = Logger::new().max_level(log::Level::Trace);
        assert!(logger.verbosity.is_none());
    }

    #[test]
    fn module_path_works() {
        let logger = Logger::new().module_path(false);
        assert!(!logger.include_module_path);
    }

    #[test]
    fn no_module_path_works() {
        let logger = Logger::new().no_module_path();
        assert!(!logger.include_module_path);
    }

    #[test]
    fn verbosity_works() {
        let logger = Logger::new().verbosity(3);
        assert_eq!(logger.verbosity, Some(3));
    }

    #[test]
    fn output_works() {
        let logger = Logger::new()
            .output(&log::Level::Error, Output::Stdout)
            .output(&log::Level::Warn, Output::Stdout)
            .output(&log::Level::Info, Output::Stderr)
            .output(&log::Level::Debug, Output::Stderr)
            .output(&log::Level::Trace, Output::Stderr);
        assert_eq!(logger.error.output, Output::Stdout);
        assert_eq!(logger.warn.output, Output::Stdout);
        assert_eq!(logger.info.output, Output::Stderr);
        assert_eq!(logger.debug.output, Output::Stderr);
        assert_eq!(logger.trace.output, Output::Stderr);
    }

    #[test]
    fn init_works() {
        let result = Logger::new().init();
        assert!(result.is_ok());
    }
}

