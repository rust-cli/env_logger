// Copyright 2014-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A simple logger configured via an environment variable which writes
//! to stdout or stderr, for use with the logging facade exposed by the
//! [`log` crate][log-crate-url].
//!
//! ## Example
//!
//! ```
//! #[macro_use] extern crate log;
//! extern crate env_logger;
//!
//! use log::Level;
//!
//! fn main() {
//!     env_logger::init();
//!
//!     debug!("this is a debug {}", "message");
//!     error!("this is printed by default");
//!
//!     if log_enabled!(Level::Info) {
//!         let x = 3 * 4; // expensive computation
//!         info!("the answer was: {}", x);
//!     }
//! }
//! ```
//!
//! Assumes the binary is `main`:
//!
//! ```{.bash}
//! $ RUST_LOG=error ./main
//! ERROR: 2017-11-01T21:37:57+00:00: main: this is printed by default
//! ```
//!
//! ```{.bash}
//! $ RUST_LOG=info ./main
//! ERROR: 2017-11-01T21:37:57+00:00: main: this is printed by default
//! INFO: 2017-11-01T21:37:57+00:00: main: the answer was: 12
//! ```
//!
//! ```{.bash}
//! $ RUST_LOG=debug ./main
//! DEBUG: 2017-11-01T21:37:57+00:00: main: this is a debug message
//! ERROR: 2017-11-01T21:37:57+00:00: main: this is printed by default
//! INFO: 2017-11-01T21:37:57+00:00: main: the answer was: 12
//! ```
//!
//! You can also set the log level on a per module basis:
//!
//! ```{.bash}
//! $ RUST_LOG=main=info ./main
//! ERROR: 2017-11-01T21:37:57+00:00: main: this is printed by default
//! INFO: 2017-11-01T21:37:57+00:00: main: the answer was: 12
//! ```
//!
//! And enable all logging:
//!
//! ```{.bash}
//! $ RUST_LOG=main ./main
//! DEBUG: 2017-11-01T21:37:57+00:00: main: this is a debug message
//! ERROR: 2017-11-01T21:37:57+00:00: main: this is printed by default
//! INFO: 2017-11-01T21:37:57+00:00: main: the answer was: 12
//! ```
//!
//! See the documentation for the [`log` crate][log-crate-url] for more
//! information about its API.
//!
//! ## Enabling logging
//!
//! Log levels are controlled on a per-module basis, and by default all logging
//! is disabled except for `error!`. Logging is controlled via the `RUST_LOG`
//! environment variable. The value of this environment variable is a
//! comma-separated list of logging directives. A logging directive is of the
//! form:
//!
//! ```text
//! path::to::module=level
//! ```
//!
//! The path to the module is rooted in the name of the crate it was compiled
//! for, so if your program is contained in a file `hello.rs`, for example, to
//! turn on logging for this file you would use a value of `RUST_LOG=hello`.
//! Furthermore, this path is a prefix-search, so all modules nested in the
//! specified module will also have logging enabled.
//!
//! The actual `level` is optional to specify. If omitted, all logging will
//! be enabled. If specified, it must be one of the strings `debug`, `error`,
//! `info`, `warn`, or `trace`.
//!
//! As the log level for a module is optional, the module to enable logging for
//! is also optional. If only a `level` is provided, then the global log
//! level for all modules is set to this value.
//!
//! Some examples of valid values of `RUST_LOG` are:
//!
//! * `hello` turns on all logging for the 'hello' module
//! * `info` turns on all info logging
//! * `hello=debug` turns on debug logging for 'hello'
//! * `hello,std::option` turns on hello, and std's option logging
//! * `error,hello=warn` turn on global error logging and also warn for hello
//!
//! ## Filtering results
//!
//! A `RUST_LOG` directive may include a regex filter. The syntax is to append `/`
//! followed by a regex. Each message is checked against the regex, and is only
//! logged if it matches. Note that the matching is done after formatting the
//! log string but before adding any logging meta-data. There is a single filter
//! for all modules.
//!
//! Some examples:
//!
//! * `hello/foo` turns on all logging for the 'hello' module where the log
//!   message includes 'foo'.
//! * `info/f.o` turns on all info logging where the log message includes 'foo',
//!   'f1o', 'fao', etc.
//! * `hello=debug/foo*foo` turns on debug logging for 'hello' where the log
//!   message includes 'foofoo' or 'fofoo' or 'fooooooofoo', etc.
//! * `error,hello=warn/[0-9]scopes` turn on global error logging and also
//!   warn for hello. In both cases the log message must include a single digit
//!   number followed by 'scopes'.
//!
//! [log-crate-url]: https://docs.rs/log/

#![doc(html_logo_url = "http://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "http://www.rust-lang.org/favicon.ico",
       html_root_url = "https://docs.rs/env_logger/0.4.3")]
#![cfg_attr(test, deny(warnings))]

// When compiled for the rustc compiler itself we want to make sure that this is
// an unstable crate
#![cfg_attr(rustbuild, feature(staged_api, rustc_private))]
#![cfg_attr(rustbuild, unstable(feature = "rustc_private", issue = "27812"))]

#![deny(missing_debug_implementations, missing_docs, warnings)]

extern crate log;
extern crate termcolor;
extern crate chrono;

use std::env;
use std::io::prelude::*;
use std::io;
use std::mem;
use std::cell::RefCell;

use chrono::format::strftime::StrftimeItems;
use log::{Log, LevelFilter, Level, Record, SetLoggerError, Metadata};
use termcolor::{ColorChoice, Color, BufferWriter};

pub mod filter;
pub mod fmt;

use self::fmt::Formatter;

/// Log target, either stdout or stderr.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Target {
    /// Logs will be sent to standard output.
    Stdout,
    /// Logs will be sent to standard error.
    Stderr,
}

/// The env logger.
///
/// This struct implements the `Log` trait from the [`log` crate][log-crate-url],
/// which allows it to act as a logger.
///
/// The [`init()`], [`try_init()`], [`Builder::init()`] and [`Builder::try_init()`]
/// methods will each construct a `Logger` and immediately initialize it as the
/// default global logger.
///
/// If you'd instead need access to the constructed `Logger`, you can use
/// [`Logger::new()`] or the associated [`Builder`] and install it with the
/// [`log` crate][log-crate-url] directly.
///
/// [log-crate-url]: https://docs.rs/log/
/// [`init()`]: fn.init.html
/// [`try_init()`]: fn.try_init.html
/// [`Builder::init()`]: struct.Builder.html#method.init
/// [`Builder::try_init()`]: struct.Builder.html#method.try_init
/// [`Logger::new()`]: #method.new
/// [`Builder`]: struct.Builder.html
pub struct Logger {
    writer: BufferWriter,
    filter: filter::Filter,
    format: Box<Fn(&mut Formatter, &Record) -> io::Result<()> + Sync + Send>,
    timestamp_format: StrftimeItems<'static>,
}

/// `Builder` acts as builder for initializing a `Logger`.
///
/// It can be used to customize the log format, change the enviromental variable used
/// to provide the logging directives and also set the default log level filter.
///
/// ## Example
///
/// ```
/// #[macro_use]
/// extern crate log;
/// extern crate env_logger;
///
/// use std::env;
/// use std::io::Write;
/// use log::LevelFilter;
/// use env_logger::Builder;
///
/// fn main() {
///     let mut builder = Builder::new();
///
///     builder.format(|buf, record| writeln!(buf, "{} - {}", record.level(), record.args()))
///            .filter(None, LevelFilter::Info);
///
///     if let Ok(rust_log) = env::var("RUST_LOG") {
///        builder.parse(&rust_log);
///     }
///
///     builder.init();
///
///     error!("error message");
///     info!("info message");
/// }
/// ```
pub struct Builder {
    filter: filter::Builder,
    format: Box<Fn(&mut Formatter, &Record) -> io::Result<()> + Sync + Send>,
    target: Target,
    timestamp_format: &'static str,
}

impl Builder {
    /// Initializes the log builder with defaults.
    pub fn new() -> Builder {
        Builder {
            filter: filter::Builder::new(),
            format: Box::new(|buf, record| {
                let ts = buf.timestamp();
                let level = record.level();
                let level_color = match level {
                    Level::Trace => Color::White,
                    Level::Debug => Color::Blue,
                    Level::Info => Color::Green,
                    Level::Warn => Color::Yellow,
                    Level::Error => Color::Red,
                };

                let write_level = write!(buf.color(level_color), "{}:", level);
                let write_args = writeln!(buf, " {}: {}: {}", ts, record.module_path(), record.args());

                write_level.and(write_args)
            }),
            target: Target::Stderr,
            timestamp_format: "%Y-%m-%dT%H:%M:%S%:z",
        }
    }

    /// Adds filters to the logger.
    ///
    /// The given module (if any) will log at most the specified level provided.
    /// If no module is provided then the filter will apply to all log messages.
    pub fn filter(&mut self,
                  module: Option<&str>,
                  level: LevelFilter) -> &mut Self {
        self.filter.filter(module, level);
        self
    }

    /// Sets the format function for formatting the log output.
    ///
    /// This function is called on each record logged and should format the
    /// log record and output it to the given [`Formatter`].
    ///
    /// The format function is expected to output the string directly to the
    /// `Formatter` so that implementations can use the [`std::fmt`] macros
    /// to format and output without intermediate heap allocations. The default
    /// `env_logger` formatter takes advantage of this.
    ///
    /// [`Formatter`]: struct.Formatter.html
    /// [`String`]: https://doc.rust-lang.org/stable/std/string/struct.String.html
    /// [`std::fmt`]: https://doc.rust-lang.org/std/fmt/index.html
    pub fn format<F: 'static>(&mut self, format: F) -> &mut Self
        where F: Fn(&mut Formatter, &Record) -> io::Result<()> + Sync + Send
    {
        self.format = Box::new(format);
        self
    }

    /// Sets the target for the log output.
    ///
    /// Env logger can log to either stdout or stderr. The default is stderr.
    pub fn target(&mut self, target: Target) -> &mut Self {
        self.target = target;
        self
    }

    /// Sets the format that the timestamp will be displayed in
    ///
    /// This should be a `&'static str` in the format parsed by
    /// [`chrono::format::strftime`].
    pub fn timestamp_format(&mut self, format: &'static str) -> &mut Self {
        self.timestamp_format = format;
        self
    }

    /// Parses the directives string in the same form as the `RUST_LOG`
    /// environment variable.
    ///
    /// See the module documentation for more details.
    pub fn parse(&mut self, filters: &str) -> &mut Self {
        self.filter.parse(filters);
        self
    }

    /// Initializes the global logger with the built env logger.
    ///
    /// This should be called early in the execution of a Rust program. Any log
    /// events that occur before initialization will be ignored.
    ///
    /// # Errors
    ///
    /// This function will fail if it is called more than once, or if another
    /// library has already initialized a global logger.
    pub fn try_init(&mut self) -> Result<(), SetLoggerError> {
        log::set_boxed_logger(|max_level| {
            let logger = self.build();
            max_level.set(logger.filter());
            Box::new(logger)
        })
    }

    /// Initializes the global logger with the built env logger.
    ///
    /// This should be called early in the execution of a Rust program. Any log
    /// events that occur before initialization will be ignored.
    ///
    /// # Panics
    ///
    /// This function will panic if it is called more than once, or if another
    /// library has already initialized a global logger.
    pub fn init(&mut self) {
        self.try_init().unwrap();
    }

    /// Build an env logger.
    pub fn build(&mut self) -> Logger {
        let writer = match self.target {
            Target::Stderr => BufferWriter::stderr(ColorChoice::Always),
            Target::Stdout => BufferWriter::stdout(ColorChoice::Always),
        };

        Logger {
            writer: writer,
            filter: self.filter.build(),
            format: mem::replace(&mut self.format, Box::new(|_, _| Ok(()))),
            timestamp_format: StrftimeItems::new(self.timestamp_format),
        }
    }
}

impl Logger {
    /// Creates a new env logger by parsing the `RUST_LOG` environment variable.
    ///
    /// The returned logger can be passed to the [`log` crate](https://docs.rs/log/)
    /// for initialization as a global logger.
    ///
    /// If you do not need to interact directly with the `Logger`, you should
    /// prefer the [`init()`] or [`try_init()`] methods, which
    /// construct a `Logger` and configure it as the default logger.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate log;
    /// extern crate env_logger;
    ///
    /// use std::env;
    /// use env_logger::Logger;
    ///
    /// fn main() {
    ///     log::set_boxed_logger(|max_level| {
    ///         let logger = Logger::new();
    ///         max_level.set(logger.filter());
    ///         Box::new(logger)
    ///     });
    /// }
    /// ```
    ///
    /// [`init()`]: fn.init.html
    /// [`try_init()`]: fn.try_init.html
    pub fn new() -> Logger {
        Self::from_env("RUST_LOG")
    }

    /// Creates a new env logger by parsing the environment variable with the
    /// given name.
    ///
    /// This is identical to the [`new()`] constructor, except it allows the
    /// name of the environment variable to be customized. For additional
    /// customization, use the [`Builder`] type instead.
    ///
    /// The returned logger can be passed to the [`log` crate](https://docs.rs/log/)
    /// for initialization as a global logger.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate log;
    /// extern crate env_logger;
    ///
    /// use std::env;
    /// use env_logger::Logger;
    ///
    /// fn main() {
    ///     log::set_boxed_logger(|max_level| {
    ///         let logger = Logger::from_env("MY_LOG");
    ///         max_level.set(logger.filter());
    ///         Box::new(logger)
    ///     });
    /// }
    /// ```
    ///
    /// [`new()`]: #method.new
    /// [`Builder`]: struct.Builder.html
    pub fn from_env(env: &str) -> Logger {
        let mut builder = Builder::new();

        if let Ok(s) = env::var(env) {
            builder.parse(&s);
        }

        builder.build()
    }

    /// Returns the maximum `LevelFilter` that this env logger instance is
    /// configured to output.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate log;
    /// extern crate env_logger;
    ///
    /// use log::LevelFilter;
    /// use env_logger::Builder;
    ///
    /// fn main() {
    ///     let mut builder = Builder::new();
    ///     builder.filter(Some("module1"), LevelFilter::Info);
    ///     builder.filter(Some("module2"), LevelFilter::Error);
    ///
    ///     let logger = builder.build();
    ///     assert_eq!(logger.filter(), LevelFilter::Info);
    /// }
    /// ```
    pub fn filter(&self) -> LevelFilter {
        self.filter.filter()
    }

    /// Checks if this record matches the configured filter.
    pub fn matches(&self, record: &Record) -> bool {
        self.filter.matches(record)
    }

    fn print(&self, formatter: &Formatter) -> io::Result<()> {
        self.writer.print(formatter.as_ref())
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.filter.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if self.matches(record) {
            // Log records are written to a thread-local buffer before being printed
            // to the terminal. We clear these buffers afterwards, but they aren't shrinked
            // so will always at least have capacity for the largest log record formatted
            // on that thread.

            thread_local! {
                static FORMATTER: RefCell<Option<Formatter>> = RefCell::new(None);
            }

            FORMATTER.with(|tl_buf| {
                let mut tl_buf = tl_buf.borrow_mut();

                if tl_buf.is_none() {
                    *tl_buf = Some(Formatter::new(self.writer.buffer(), self.timestamp_format.clone()));
                }

                // The format is guaranteed to be `Some` by this point
                let mut formatter = tl_buf.as_mut().unwrap();

                let _ = (self.format)(&mut formatter, record).and_then(|_| self.print(formatter));

                // Always clear the buffer afterwards
                formatter.clear();
            });
        }
    }

    fn flush(&self) {}
}

mod std_fmt_impls {
    use std::fmt;
    use super::*;

    impl fmt::Debug for Logger{
        fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result {
            f.debug_struct("Logger")
                .field("filter", &self.filter)
                .finish()
        }
    }

    impl fmt::Debug for Builder{
        fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result {
            f.debug_struct("Logger")
            .field("filter", &self.filter)
            .field("target", &self.target)
            .finish()
        }
    }
}

/// Attempts to initialize the global logger with an env logger.
///
/// This should be called early in the execution of a Rust program. Any log
/// events that occur before initialization will be ignored.
///
/// # Errors
///
/// This function will fail if it is called more than once, or if another
/// library has already initialized a global logger.
pub fn try_init() -> Result<(), SetLoggerError> {
    try_init_from_env("RUST_LOG")
}

/// Initializes the global logger with an env logger.
///
/// This should be called early in the execution of a Rust program. Any log
/// events that occur before initialization will be ignored.
///
/// # Panics
///
/// This function will panic if it is called more than once, or if another
/// library has already initialized a global logger.
pub fn init() {
    try_init().unwrap();
}

/// Attempts to initialize the global logger with an env logger from the given
/// environment variable.
///
/// This should be called early in the execution of a Rust program. Any log
/// events that occur before initialization will be ignored.
///
/// # Errors
///
/// This function will fail if it is called more than once, or if another
/// library has already initialized a global logger.
pub fn try_init_from_env(env: &str) -> Result<(), SetLoggerError> {
    let mut builder = Builder::new();

    if let Ok(s) = env::var(env) {
        builder.parse(&s);
    }

    builder.try_init()
}

/// Initializes the global logger with an env logger from the given environment
/// variable.
///
/// This should be called early in the execution of a Rust program. Any log
/// events that occur before initialization will be ignored.
///
/// # Panics
///
/// This function will panic if it is called more than once, or if another
/// library has already initialized a global logger.
pub fn init_from_env(env: &str) {
    try_init_from_env(env).unwrap();
}
