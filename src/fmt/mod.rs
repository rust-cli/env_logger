//! Formatting for log records.
//!
//! This module contains a [`Formatter`] that can be used to format log records
//! into without needing temporary allocations. Usually you won't need to worry
//! about the contents of this module and can use the `Formatter` like an ordinary
//! [`Write`].
//!
//! # Formatting log records
//!
//! The format used to print log records can be customised using the [`Builder::format`]
//! method.
//! Custom formats can apply different color and weight to printed values using
//! [`Style`] builders.
//!
//! ```
//! use std::io::Write;
//!
//! let mut builder = env_logger::Builder::new();
//!
//! builder.format(|buf, record| {
//!     writeln!(buf, "{}: {}",
//!         record.level(),
//!         record.args())
//! });
//! ```
//!
//! [`Formatter`]: struct.Formatter.html
//! [`Style`]: struct.Style.html
//! [`Builder::format`]: ../struct.Builder.html#method.format
//! [`Write`]: https://doc.rust-lang.org/stable/std/io/trait.Write.html

use std::io::prelude::*;
use std::{io, fmt};
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Display;

use log::Record;

mod termcolor;
mod atty;
mod humantime;

use self::termcolor::{Buffer, BufferWriter};
use self::atty::{is_stdout, is_stderr};

pub use self::humantime::pub_use_in_super::*;
pub use self::termcolor::pub_use_in_super::*;

pub(super) mod pub_use_in_super {
    pub use super::{Target, WriteStyle, Formatter};

    #[cfg(feature = "termcolor")]
    pub use super::Color;
}

pub(super) struct Format {
    pub default_format_timestamp: bool,
    pub default_format_module_path: bool,
    pub default_format_level: bool,
    pub default_format_timestamp_nanos: bool,
    pub custom_format: Option<Box<Fn(&mut Formatter, &Record) -> io::Result<()> + Sync + Send>>,
}

impl Default for Format {
    fn default() -> Self {
        Format {
            default_format_timestamp: true,
            default_format_module_path: true,
            default_format_level: true,
            default_format_timestamp_nanos: false,
            custom_format: None,
        }
    }
}

impl Format {
    /// Convert the format into a callable function.
    /// 
    /// If the `custom_format` is `Some`, then any `default_format` switches are ignored.
    /// If the `custom_format` is `None`, then a default format is returned.
    /// Any `default_format` switches set to `false` won't be written by the format.
    pub fn into_boxed_fn(self) -> Box<Fn(&mut Formatter, &Record) -> io::Result<()> + Sync + Send> {
        if let Some(fmt) = self.custom_format {
            fmt
        }
        else {
            Box::new(move |buf, record| {
                let fmt = DefaultFormat {
                    timestamp: self.default_format_level,
                    module_path: self.default_format_module_path,
                    level: self.default_format_level,
                    timestamp_nanos: self.default_format_timestamp_nanos,
                    written_header_value: false,
                    buf,
                };

                fmt.write(record)
            })
        }
    }
}

#[cfg(feature = "termcolor")]
type SubtleStyle = StyledValue<'static, &'static str>;
#[cfg(not(feature = "termcolor"))]
type SubtleStyle = &'static str;

struct DefaultFormat<'a> {
    timestamp: bool,
    module_path: bool,
    level: bool,
    timestamp_nanos: bool,
    written_header_value: bool,
    buf: &'a mut Formatter,
}

impl<'a> DefaultFormat<'a> {
    fn write(mut self, record: &Record) -> io::Result<()> {
        self.write_level(record)?;
        self.write_timestamp()?;
        self.write_module_path(record)?;
        self.finish_header()?;

        self.write_args(record)
    }

    fn subtle_style(&self, text: &'static str) -> SubtleStyle {
        #[cfg(feature = "termcolor")]
        {
            self.buf.style()
                .set_color(Color::Black)
                .set_intense(true)
                .into_value(text)
        }
        #[cfg(not(feature = "termcolor"))]
        {
            text
        }
    }

    fn write_header_value<T>(&mut self, value: T) -> io::Result<()>
    where
        T: Display,
    {
        if !self.written_header_value {
            self.written_header_value = true;

            let open_brace = self.subtle_style("[");
            write!(self.buf, "{}{}", open_brace, value)
        } else {
            write!(self.buf, " {}", value)
        }
    }

    fn write_level(&mut self, record: &Record) -> io::Result<()> {
        if !self.level {
            return Ok(())
        }

        let level = {
            #[cfg(feature = "termcolor")]
            {
                self.buf.default_styled_level(record.level())
            }
            #[cfg(not(feature = "termcolor"))]
            {
                record.level()
            }
        };

        self.write_header_value(format_args!("{:<5}", level))
    }

    fn write_timestamp(&mut self) -> io::Result<()> {
        #[cfg(feature = "humantime")]
        {
            if !self.timestamp {
                return Ok(())
            }

            if self.timestamp_nanos {
                let ts_nanos = self.buf.precise_timestamp();
                self.write_header_value(ts_nanos)
            } else {
                let ts = self.buf.timestamp();
                self.write_header_value(ts)
            }
        }
        #[cfg(not(feature = "humantime"))]
        {
            let _ = self.timestamp;
            let _ = self.timestamp_nanos;
            Ok(())
        }
    }

    fn write_module_path(&mut self, record: &Record) -> io::Result<()> {
        if !self.module_path {
            return Ok(())
        }

        if let Some(module_path) = record.module_path() {
            self.write_header_value(module_path)
        } else {
            Ok(())
        }
    }

    fn finish_header(&mut self) -> io::Result<()> {
        if self.written_header_value {
            let close_brace = self.subtle_style("]");
            write!(self.buf, "{}", close_brace)
        } else {
            Ok(())
        }
    }

    fn write_args(&mut self, record: &Record) -> io::Result<()> {
        writeln!(self.buf, "{}", record.args())
    }
}

/// A formatter to write logs into.
///
/// `Formatter` implements the standard [`Write`] trait for writing log records.
/// It also supports terminal colors, through the [`style`] method.
///
/// # Examples
///
/// Use the [`writeln`] macro to easily format a log record:
///
/// ```
/// use std::io::Write;
///
/// let mut builder = env_logger::Builder::new();
///
/// builder.format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()));
/// ```
///
/// [`Write`]: https://doc.rust-lang.org/stable/std/io/trait.Write.html
/// [`writeln`]: https://doc.rust-lang.org/stable/std/macro.writeln.html
/// [`style`]: #method.style
pub struct Formatter {
    buf: Rc<RefCell<Buffer>>,
    write_style: WriteStyle,
}

/// Log target, either `stdout` or `stderr`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Target {
    /// Logs will be sent to standard output.
    Stdout,
    /// Logs will be sent to standard error.
    Stderr,
}

impl Default for Target {
    fn default() -> Self {
        Target::Stderr
    }
}

/// Whether or not to print styles to the target.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WriteStyle {
    /// Try to print styles, but don't force the issue.
    Auto,
    /// Try very hard to print styles.
    Always,
    /// Never print styles.
    Never,
}

impl Default for WriteStyle {
    fn default() -> Self {
        WriteStyle::Auto
    }
}

/// A terminal target with color awareness.
pub(crate) struct Writer {
    inner: BufferWriter,
    write_style: WriteStyle,
}

impl Writer {
    pub(crate) fn write_style(&self) -> WriteStyle {
        self.write_style
    }
}

/// A builder for a terminal writer.
///
/// The target and style choice can be configured before building.
pub(crate) struct Builder {
    target: Target,
    write_style: WriteStyle,
}

impl Builder {
    /// Initialize the writer builder with defaults.
    pub fn new() -> Self {
        Builder {
            target: Default::default(),
            write_style: Default::default(),
        }
    }

    /// Set the target to write to.
    pub fn target(&mut self, target: Target) -> &mut Self {
        self.target = target;
        self
    }

    /// Parses a style choice string.
    ///
    /// See the [Disabling colors] section for more details.
    ///
    /// [Disabling colors]: ../index.html#disabling-colors
    pub fn parse(&mut self, write_style: &str) -> &mut Self {
        self.write_style(parse_write_style(write_style))
    }

    /// Whether or not to print style characters when writing.
    pub fn write_style(&mut self, write_style: WriteStyle) -> &mut Self {
        self.write_style = write_style;
        self
    }

    /// Build a terminal writer.
    pub fn build(&mut self) -> Writer {
        let color_choice = match self.write_style {
            WriteStyle::Auto => {
                if match self.target {
                    Target::Stderr => is_stderr(),
                    Target::Stdout => is_stdout(),
                } {
                    WriteStyle::Auto
                } else {
                    WriteStyle::Never
                }
            },
            color_choice => color_choice,
        };

        let writer = match self.target {
            Target::Stderr => BufferWriter::stderr(color_choice),
            Target::Stdout => BufferWriter::stdout(color_choice),
        };

        Writer {
            inner: writer,
            write_style: self.write_style,
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder::new()
    }
}

impl Formatter {
    pub(crate) fn new(writer: &Writer) -> Self {
        Formatter {
            buf: Rc::new(RefCell::new(writer.inner.buffer())),
            write_style: writer.write_style(),
        }
    }

    pub(crate) fn write_style(&self) -> WriteStyle {
        self.write_style
    }

    pub(crate) fn print(&self, writer: &Writer) -> io::Result<()> {
        writer.inner.print(&self.buf.borrow())
    }

    pub(crate) fn clear(&mut self) {
        self.buf.borrow_mut().clear()
    }
}

impl Write for Formatter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buf.borrow_mut().flush()
    }
}

impl fmt::Debug for Writer {
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result {
        f.debug_struct("Writer").finish()
    }
}

impl fmt::Debug for Formatter {
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result {
        f.debug_struct("Formatter").finish()
    }
}

impl fmt::Debug for Builder {
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result {
        f.debug_struct("Logger")
        .field("target", &self.target)
        .field("write_style", &self.write_style)
        .finish()
    }
}

fn parse_write_style(spec: &str) -> WriteStyle {
    match spec {
        "auto" => WriteStyle::Auto,
        "always" => WriteStyle::Always,
        "never" => WriteStyle::Never,
        _ => Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_write_style_valid() {
        let inputs = vec![
            ("auto", WriteStyle::Auto),
            ("always", WriteStyle::Always),
            ("never", WriteStyle::Never),
        ];

        for (input, expected) in inputs {
            assert_eq!(expected, parse_write_style(input));
        }
    }

    #[test]
    fn parse_write_style_invalid() {
        let inputs = vec![
            "",
            "true",
            "false",
            "NEVER!!"
        ];

        for input in inputs {
            assert_eq!(WriteStyle::Auto, parse_write_style(input));
        }
    }
}