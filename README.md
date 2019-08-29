# loggerv

[![build status](https://secure.travis-ci.org/clux/loggerv.svg)](http://travis-ci.org/clux/loggerv)
[![coverage status](http://img.shields.io/coveralls/clux/loggerv.svg)](https://coveralls.io/r/clux/loggerv)
[![crates status](https://img.shields.io/crates/v/loggerv.svg)](https://crates.io/crates/loggerv)

A simple `stdout` and `stderr` writing `Logger` implementation of the `log` crate, using `ansi_term` for colors and configured via a log level. Designed for simple Command Line Interfaces (CLIs).

## Usage

First, add this to your `Cargo.toml`:

```toml
[dependencies]
log = "0.4"
loggerv = "0.7"
```

Next, add this to the `main.rs` or the file containing the `main` function for your CLI program:

```rust
extern crate loggerv;

```

## Getting Started

Clone this repository, then run the following commands to see the log level change:

```bash
$ cargo run --example quick
$ cargo run --example quick -- -v
$ cargo run --example quick -- -vv
$ cargo run --example quick -- -vvv
```

This will run an example that uses the [clap](https://crates.io/crates/clap) argument parser to change the log level at run-time based on the number of `-v` arguments that are passed to the application. As the occurrence of the `-v` argument increases, the number of log statements that are displayed should increase.

Next, run the following commands:

```bash
$ cargo run --example compile-time-config
```

This will run an example that changes the output from defaults at compile-time. The following commands will demonstration configuration at run-time:

```bash
$ cargo run --example run-time-config
$ cargo run --example run-time-config -- -v
$ cargo run --example run-time-config -- -vv
$ cargo run --example run-time-config -- -vvv
$ cargo run --example run-time-config -- -vvv
$ cargo run --example run-time-config -- -vvv -l
$ cargo run --example run-time-config -- -vvv -l -d
$ cargo run --example run-time-config -- -vvv -l -d --no-module-path
$ cargo run --example run-time-config -- -vvv -l -d --no-module-path --no-color
```

Similar to the quick example, as the occurrence of the `-v` argument increases, the number of log statements that are displayed should increase. As the various configuration arguments, i.e. `-l`, `-d`, etc. are added, the format of the log statements change. The `-h,--help` flag can be used to display information about the various flags and their effects on logging and output.

Finally, run the following commands to demonstration build profile configuration:

```bash
$ cargo run --example cfg-config
$ cargo run --release --example cfg-config
```

The number of log statements are displayed based on the build profile, either Debug or Release.

## [Documentation](https://docs.rs/loggerv)

## License

MIT-Licensed. See LICENSE file for details.

