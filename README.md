# loggerv

[![build status](https://secure.travis-ci.org/clux/loggerv.svg)](http://travis-ci.org/clux/loggerv)
[![coverage status](http://img.shields.io/coveralls/clux/loggerv.svg)](https://coveralls.io/r/clux/loggerv)
[![crates status](https://img.shields.io/crates/v/loggerv.svg)](https://crates.io/crates/loggerv)

A simple `stdout` and `stderr` writing `Logger` implementation of the `log` crate, using `ansi_term` for colors and configured via a log level. Designed for simple Command Line Interfaces (CLIs).

## Usage

First, add this to your `Cargo.toml`:

```toml
[dependencies]
log = "0.3"
loggerv = "0.4"
```

Next, add this to the `main.rs` or the file containing the `main` function for your CLI program:

```rust
extern crate loggerv;

```

## Getting Started

Clone this repository, then run the following commands to see the log level change:

```
$ cargo run --example clap
$ cargo run --example clap -- -v
$ cargo run --example clap -- -vv
$ cargo run --example clap -- -vvv
```

This will run an example that uses the [clap](https://crates.io/crates/clap) argument parser to change the log level at run-time based on the number of `-v` arguments that are passed to the application. As the occurrence of the `-v` argument increases, the number of log statements that are displayed should increase.

Next, run the following commands:

```
$ cargo run --example config
$ cargo run --example config -- -v
$ cargo run --example config -- -vv
$ cargo run --example config -- -vvv
$ cargo run --example config -- -vvv
$ cargo run --example config -- -vvv -l
$ cargo run --example config -- -vvv -l -d
$ cargo run --example config -- -vvv -l -d -n
```

Similar to the previous example, as the occurrence of the `-v` argument increases, the number of log statements that are displayed should increase. As the various configuration arguments, i.e. `-l`, `-d`, and `-n`, are added, the format of the log statements change. 

For both examples, the `-h,--help` flag can be used to display information about the various flags and their effects on logging and output.

## [Documentation](http://clux.github.io/loggerv)

## License

MIT-Licensed. See LICENSE file for details.

