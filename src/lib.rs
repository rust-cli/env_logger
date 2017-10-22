// Copyright 2014-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A logger configured via an environment variable which writes to standard
//! error, for use with the logging facade exposed by the
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
//! ERROR:main: this is printed by default
//! ```
//!
//! ```{.bash}
//! $ RUST_LOG=info ./main
//! ERROR:main: this is printed by default
//! INFO:main: the answer was: 12
//! ```
//!
//! ```{.bash}
//! $ RUST_LOG=debug ./main
//! DEBUG:main: this is a debug message
//! ERROR:main: this is printed by default
//! INFO:main: the answer was: 12
//! ```
//!
//! You can also set the log level on a per module basis:
//!
//! ```{.bash}
//! $ RUST_LOG=main=info ./main
//! ERROR:main: this is printed by default
//! INFO:main: the answer was: 12
//! ```
//!
//! And enable all logging:
//!
//! ```{.bash}
//! $ RUST_LOG=main ./main
//! DEBUG:main: this is a debug message
//! ERROR:main: this is printed by default
//! INFO:main: the answer was: 12
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
//! ## Using `env_logger` in your own logger
//!
//! You can use `env_logger`'s filtering functionality with your own logger by
//! calling [`Logger::matches`](struct.Logger.html#method.matches).
//!
//! ```
//! extern crate log;
//! extern crate env_logger;
//! use env_logger::Logger as EnvLogger;
//! use log::{Log, Metadata, Record};
//!
//! struct MyLogger {
//!     filter: EnvLogger
//! }
//!
//! impl MyLogger {
//!     fn new() -> MyLogger {
//!         use env_logger::{Builder, Target};
//!         let mut builder = Builder::new();
//!         builder.target(Target::Silent);
//!
//!         if let Ok(ref filter) = std::env::var("MY_LOG_LEVEL") {
//!            builder.parse(filter);
//!         }
//!
//!         MyLogger {
//!             filter: builder.build()
//!         }
//!     }
//! }
//!
//! impl Log for MyLogger {
//!     fn enabled(&self, metadata: &Metadata) -> bool {
//!         self.filter.enabled(metadata)
//!     }
//!
//!     fn log(&self, record: &Record) {
//!         if !self.filter.matches(record) {
//!             return;
//!         }
//!
//!         println!("{:?}", record)
//!     }
//!
//!     fn flush(&self) {}
//! }
//! # fn main() {}
//! ```
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

use std::env;
use std::io::prelude::*;
use std::io;
use std::mem;
use std::fmt;

use log::{Log, Level, LevelFilter, Record, SetLoggerError, Metadata};

#[cfg(feature = "regex")]
#[path = "regex.rs"]
mod filter;

#[cfg(not(feature = "regex"))]
#[path = "string.rs"]
mod filter;

/// Log target, either stdout or stderr.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Target {
    /// Logs will be sent to standard output.
    Stdout,
    /// Logs will be sent to standard error.
    Stderr,
    /// Logs will be silenced.
    Silent,
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
    directives: Vec<Directive>,
    filter: Option<filter::Filter>,
    format: Box<Fn(&mut Write, &Record) -> io::Result<()> + Sync + Send>,
    target: Target,
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
/// use std::io;
/// use log::{Record, LevelFilter};
/// use env_logger::Builder;
///
/// fn main() {
///     let format = |buf: &mut io::Write, record: &Record| {
///         writeln!(buf, "{} - {}", record.level(), record.args())
///     };
///
///     let mut builder = Builder::new();
///     builder.format(format).filter(None, LevelFilter::Info);
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
    directives: Vec<Directive>,
    filter: Option<filter::Filter>,
    format: Box<Fn(&mut Write, &Record) -> io::Result<()> + Sync + Send>,
    target: Target,
}

impl Builder {
    /// Initializes the log builder with defaults.
    pub fn new() -> Builder {
        Builder {
            directives: Vec::new(),
            filter: None,
            format: Box::new(|buf: &mut Write, record: &Record| {
                writeln!(buf, "{}:{}: {}", record.level(),
                         record.module_path(), record.args())
            }),
            target: Target::Stderr,
        }
    }

    /// Adds filters to the logger.
    ///
    /// The given module (if any) will log at most the specified level provided.
    /// If no module is provided then the filter will apply to all log messages.
    pub fn filter(&mut self,
                  module: Option<&str>,
                  level: LevelFilter) -> &mut Self {
        self.directives.push(Directive {
            name: module.map(|s| s.to_string()),
            level: level,
        });
        self
    }

    /// Sets the format function for formatting the log output.
    ///
    /// This function is called on each record logged and should format the
    /// log record and output it to the given [`Write`] trait object.
    ///
    /// The format function is expected to output the string directly to the
    /// [`Write`] trait object (rather than returning a [`String`]), so that
    /// implementations can use the [`std::fmt`] macros to format and output
    /// without intermediate heap allocations. The default `env_logger`
    /// formatter takes advantage of this.
    ///
    /// [`Write`]: https://doc.rust-lang.org/stable/std/io/trait.Write.html
    /// [`String`]: https://doc.rust-lang.org/stable/std/string/struct.String.html
    /// [`std::fmt`]: https://doc.rust-lang.org/std/fmt/index.html
    pub fn format<F: 'static>(&mut self, format: F) -> &mut Self
        where F: Fn(&mut Write, &Record) -> io::Result<()> + Sync + Send
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

    /// Parses the directives string in the same form as the `RUST_LOG`
    /// environment variable.
    ///
    /// See the module documentation for more details.
    pub fn parse(&mut self, filters: &str) -> &mut Self {
        let (directives, filter) = parse_spec(filters);

        self.filter = filter;

        for directive in directives {
            self.directives.push(directive);
        }
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
        if self.directives.is_empty() {
            // Adds the default filter if none exist
            self.directives.push(Directive {
                name: None,
                level: LevelFilter::Error,
            });
        } else {
            // Sort the directives by length of their name, this allows a
            // little more efficient lookup at runtime.
            self.directives.sort_by(|a, b| {
                let alen = a.name.as_ref().map(|a| a.len()).unwrap_or(0);
                let blen = b.name.as_ref().map(|b| b.len()).unwrap_or(0);
                alen.cmp(&blen)
            });
        }

        Logger {
            directives: mem::replace(&mut self.directives, Vec::new()),
            filter: mem::replace(&mut self.filter, None),
            format: mem::replace(&mut self.format, Box::new(|_, _| Ok(()))),
            target: mem::replace(&mut self.target, Target::Stderr),
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
        self.directives.iter()
            .map(|d| d.level)
            .max()
            .unwrap_or(LevelFilter::Off)
    }

    /// Checks if this record matches the configured filter.
    pub fn matches(&self, record: &Record) -> bool {
        if !Log::enabled(self, record.metadata()) {
            return false;
        }

        if let Some(filter) = self.filter.as_ref() {
            if !filter.is_match(&*record.args().to_string()) {
                return false;
            }
        }

        true
    }

    fn enabled(&self, level: Level, target: &str) -> bool {
        // Search for the longest match, the vector is assumed to be pre-sorted.
        for directive in self.directives.iter().rev() {
            match directive.name {
                Some(ref name) if !target.starts_with(&**name) => {},
                Some(..) | None => {
                    return level <= directive.level
                }
            }
        }
        false
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.enabled(metadata.level(), metadata.target())
    }

    fn log(&self, record: &Record) {
        let _ = match self.target {
            Target::Stdout if self.matches(record) => (self.format)(&mut io::stdout(), record),
            Target::Stderr if self.matches(record) => (self.format)(&mut io::stderr(), record),
            _ => Ok(()),
        };
    }

    fn flush(&self) {}
}

impl fmt::Debug for Logger{
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result{
        f.debug_struct("Logger")
            .field("directives", &self.directives)
            .field("filter", &self.filter)
            .field("target", &self.target)
            .finish()
    }
}

impl fmt::Debug for Builder{
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result{
        write!(f,"Logger{{directives:{:?}, filter:{:?}, target:{:?} }}", self.directives, self.filter, self.target)
    }
}

#[derive(Debug)]
struct Directive {
    name: Option<String>,
    level: LevelFilter,
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

/// Parse a logging specification string (e.g: "crate1,crate2::mod3,crate3::x=error/foo")
/// and return a vector with log directives.
fn parse_spec(spec: &str) -> (Vec<Directive>, Option<filter::Filter>) {
    let mut dirs = Vec::new();

    let mut parts = spec.split('/');
    let mods = parts.next();
    let filter = parts.next();
    if parts.next().is_some() {
        println!("warning: invalid logging spec '{}', \
                 ignoring it (too many '/'s)", spec);
        return (dirs, None);
    }
    mods.map(|m| { for s in m.split(',') {
        if s.len() == 0 { continue }
        let mut parts = s.split('=');
        let (log_level, name) = match (parts.next(), parts.next().map(|s| s.trim()), parts.next()) {
            (Some(part0), None, None) => {
                // if the single argument is a log-level string or number,
                // treat that as a global fallback
                match part0.parse() {
                    Ok(num) => (num, None),
                    Err(_) => (LevelFilter::max(), Some(part0)),
                }
            }
            (Some(part0), Some(""), None) => (LevelFilter::max(), Some(part0)),
            (Some(part0), Some(part1), None) => {
                match part1.parse() {
                    Ok(num) => (num, Some(part0)),
                    _ => {
                        println!("warning: invalid logging spec '{}', \
                                 ignoring it", part1);
                        continue
                    }
                }
            },
            _ => {
                println!("warning: invalid logging spec '{}', \
                         ignoring it", s);
                continue
            }
        };
        dirs.push(Directive {
            name: name.map(|s| s.to_string()),
            level: log_level,
        });
    }});

    let filter = filter.map_or(None, |filter| {
        match filter::Filter::new(filter) {
            Ok(re) => Some(re),
            Err(e) => {
                println!("warning: invalid regex filter - {}", e);
                None
            }
        }
    });

    return (dirs, filter);
}

#[cfg(test)]
mod tests {
    use log::{Level, LevelFilter};

    use super::{Builder, Logger, Directive, parse_spec};

    fn make_logger(dirs: Vec<Directive>) -> Logger {
        let mut logger = Builder::new().build();
        logger.directives = dirs;
        logger
    }

    #[test]
    fn filter_info() {
        let logger = Builder::new().filter(None, LevelFilter::Info).build();
        assert!(logger.enabled(Level::Info, "crate1"));
        assert!(!logger.enabled(Level::Debug, "crate1"));
    }

    #[test]
    fn filter_beginning_longest_match() {
        let logger = Builder::new()
                        .filter(Some("crate2"), LevelFilter::Info)
                        .filter(Some("crate2::mod"), LevelFilter::Debug)
                        .filter(Some("crate1::mod1"), LevelFilter::Warn)
                        .build();
        assert!(logger.enabled(Level::Debug, "crate2::mod1"));
        assert!(!logger.enabled(Level::Debug, "crate2"));
    }

    #[test]
    fn parse_default() {
        let logger = Builder::new().parse("info,crate1::mod1=warn").build();
        assert!(logger.enabled(Level::Warn, "crate1::mod1"));
        assert!(logger.enabled(Level::Info, "crate2::mod2"));
    }

    #[test]
    fn match_full_path() {
        let logger = make_logger(vec![
            Directive {
                name: Some("crate2".to_string()),
                level: LevelFilter::Info
            },
            Directive {
                name: Some("crate1::mod1".to_string()),
                level: LevelFilter::Warn
            }
        ]);
        assert!(logger.enabled(Level::Warn, "crate1::mod1"));
        assert!(!logger.enabled(Level::Info, "crate1::mod1"));
        assert!(logger.enabled(Level::Info, "crate2"));
        assert!(!logger.enabled(Level::Debug, "crate2"));
    }

    #[test]
    fn no_match() {
        let logger = make_logger(vec![
            Directive { name: Some("crate2".to_string()), level: LevelFilter::Info },
            Directive { name: Some("crate1::mod1".to_string()), level: LevelFilter::Warn }
        ]);
        assert!(!logger.enabled(Level::Warn, "crate3"));
    }

    #[test]
    fn match_beginning() {
        let logger = make_logger(vec![
            Directive { name: Some("crate2".to_string()), level: LevelFilter::Info },
            Directive { name: Some("crate1::mod1".to_string()), level: LevelFilter::Warn }
        ]);
        assert!(logger.enabled(Level::Info, "crate2::mod1"));
    }

    #[test]
    fn match_beginning_longest_match() {
        let logger = make_logger(vec![
            Directive { name: Some("crate2".to_string()), level: LevelFilter::Info },
            Directive { name: Some("crate2::mod".to_string()), level: LevelFilter::Debug },
            Directive { name: Some("crate1::mod1".to_string()), level: LevelFilter::Warn }
        ]);
        assert!(logger.enabled(Level::Debug, "crate2::mod1"));
        assert!(!logger.enabled(Level::Debug, "crate2"));
    }

    #[test]
    fn match_default() {
        let logger = make_logger(vec![
            Directive { name: None, level: LevelFilter::Info },
            Directive { name: Some("crate1::mod1".to_string()), level: LevelFilter::Warn }
        ]);
        assert!(logger.enabled(Level::Warn, "crate1::mod1"));
        assert!(logger.enabled(Level::Info, "crate2::mod2"));
    }

    #[test]
    fn zero_level() {
        let logger = make_logger(vec![
            Directive { name: None, level: LevelFilter::Info },
            Directive { name: Some("crate1::mod1".to_string()), level: LevelFilter::Off }
        ]);
        assert!(!logger.enabled(Level::Error, "crate1::mod1"));
        assert!(logger.enabled(Level::Info, "crate2::mod2"));
    }

    #[test]
    fn parse_spec_valid() {
        let (dirs, filter) = parse_spec("crate1::mod1=error,crate1::mod2,crate2=debug");
        assert_eq!(dirs.len(), 3);
        assert_eq!(dirs[0].name, Some("crate1::mod1".to_string()));
        assert_eq!(dirs[0].level, LevelFilter::Error);

        assert_eq!(dirs[1].name, Some("crate1::mod2".to_string()));
        assert_eq!(dirs[1].level, LevelFilter::max());

        assert_eq!(dirs[2].name, Some("crate2".to_string()));
        assert_eq!(dirs[2].level, LevelFilter::Debug);
        assert!(filter.is_none());
    }

    #[test]
    fn parse_spec_invalid_crate() {
        // test parse_spec with multiple = in specification
        let (dirs, filter) = parse_spec("crate1::mod1=warn=info,crate2=debug");
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].name, Some("crate2".to_string()));
        assert_eq!(dirs[0].level, LevelFilter::Debug);
        assert!(filter.is_none());
    }

    #[test]
    fn parse_spec_invalid_level() {
        // test parse_spec with 'noNumber' as log level
        let (dirs, filter) = parse_spec("crate1::mod1=noNumber,crate2=debug");
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].name, Some("crate2".to_string()));
        assert_eq!(dirs[0].level, LevelFilter::Debug);
        assert!(filter.is_none());
    }

    #[test]
    fn parse_spec_string_level() {
        // test parse_spec with 'warn' as log level
        let (dirs, filter) = parse_spec("crate1::mod1=wrong,crate2=warn");
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].name, Some("crate2".to_string()));
        assert_eq!(dirs[0].level, LevelFilter::Warn);
        assert!(filter.is_none());
    }

    #[test]
    fn parse_spec_empty_level() {
        // test parse_spec with '' as log level
        let (dirs, filter) = parse_spec("crate1::mod1=wrong,crate2=");
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].name, Some("crate2".to_string()));
        assert_eq!(dirs[0].level, LevelFilter::max());
        assert!(filter.is_none());
    }

    #[test]
    fn parse_spec_global() {
        // test parse_spec with no crate
        let (dirs, filter) = parse_spec("warn,crate2=debug");
        assert_eq!(dirs.len(), 2);
        assert_eq!(dirs[0].name, None);
        assert_eq!(dirs[0].level, LevelFilter::Warn);
        assert_eq!(dirs[1].name, Some("crate2".to_string()));
        assert_eq!(dirs[1].level, LevelFilter::Debug);
        assert!(filter.is_none());
    }

    #[test]
    fn parse_spec_valid_filter() {
        let (dirs, filter) = parse_spec("crate1::mod1=error,crate1::mod2,crate2=debug/abc");
        assert_eq!(dirs.len(), 3);
        assert_eq!(dirs[0].name, Some("crate1::mod1".to_string()));
        assert_eq!(dirs[0].level, LevelFilter::Error);

        assert_eq!(dirs[1].name, Some("crate1::mod2".to_string()));
        assert_eq!(dirs[1].level, LevelFilter::max());

        assert_eq!(dirs[2].name, Some("crate2".to_string()));
        assert_eq!(dirs[2].level, LevelFilter::Debug);
        assert!(filter.is_some() && filter.unwrap().to_string() == "abc");
    }

    #[test]
    fn parse_spec_invalid_crate_filter() {
        let (dirs, filter) = parse_spec("crate1::mod1=error=warn,crate2=debug/a.c");
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].name, Some("crate2".to_string()));
        assert_eq!(dirs[0].level, LevelFilter::Debug);
        assert!(filter.is_some() && filter.unwrap().to_string() == "a.c");
    }

    #[test]
    fn parse_spec_empty_with_filter() {
        let (dirs, filter) = parse_spec("crate1/a*c");
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].name, Some("crate1".to_string()));
        assert_eq!(dirs[0].level, LevelFilter::max());
        assert!(filter.is_some() && filter.unwrap().to_string() == "a*c");
    }
}
