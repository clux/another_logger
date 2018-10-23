//! An example using the Builder pattern API to configure the logger at compile-time.
//!
//! The default output is `module::path: message`, and the "tag", which is the text to the left of
//! the colon, is colorized. This example shows how to change the output to: `level: message` with
//! no colorization. It also demonstrates explicitly setting the log level with the `max_level`
//! instead of implicitly with a verbosity.
//!
//! The [clap](https://crates.io/crates/clap) argument parser is used in this example, but loggerv
//! works with any argument parser.

extern crate ansi_term;
#[macro_use] extern crate log;
extern crate loggerv;
use loggerv::{Logger, ColorMode};

fn main() {
    // Add the following line near the beginning of the main function for an application to enable
    // colorized output on Windows 10.
    //
    // Based on documentation for the ansi_term crate, Windows 10 supports ANSI escape characters,
    // but it must be enabled first using the `ansi_term::enable_ansi_support()` function. It is
    // conditionally compiled and only exists for Windows builds. To avoid build errors on
    // non-windows platforms, a cfg guard should be put in place.
    #[cfg(windows)] ansi_term::enable_ansi_support().unwrap();

    Logger::new()
        .max_level(log::Level::Info)
        .level(true)
        .no_module_path()
        .colors(ColorMode::Always)
        .init()
        .unwrap();

    error!("This is printed to stderr with this configuration");
    warn!("This is printed to stderr with this configuration");
    info!("This is printed to stdout with this configuration");
    debug!("This is not printed to stdout with this configuration");
    trace!("This is not printed to stdout with this configuration");
}

