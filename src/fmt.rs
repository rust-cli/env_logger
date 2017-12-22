//! Formatting for log records.
//! 
//! This module contains a [`Formatter`] that can be used to format log records
//! into without needing temporary allocations. Usually you won't need to worry
//! about the contents of this module and can use the `Formatter` like an ordinary
//! [`Write`].
//! 
//! [`Formatter`]: struct.Formatter.html
//! [`Write`]: https://doc.rust-lang.org/stable/std/io/trait.Write.html

use std::io::prelude::*;
use std::io;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

use termcolor::{ColorSpec, Buffer, BufferWriter, WriteColor};
use chrono::{DateTime, Utc};
use chrono::format::Item;

pub use termcolor::Color;

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
    write_style: bool
}

/// A set of styles to apply to the terminal output.
/// 
/// Call [`Formatter.style`] to get a `Style` and use the builder methods to 
/// set styling properties, like [color] and [weight].
/// To print a value using the style, wrap it in a call to [`value`] when the log
/// record is formatted.
/// 
/// # Examples
/// 
/// Create a bold, red colored style and use it to print the log level:
/// 
/// ```
/// use std::io::Write;
/// use env_logger::fmt::Color;
/// 
/// let mut builder = env_logger::Builder::new();
/// 
/// builder.format(|buf, record| {
///     let mut level_style = buf.style();
/// 
///     level_style.set_color(Color::Red).set_bold(true);
/// 
///     writeln!(buf, "{}: {}",
///         level_style.value(record.level()),
///         record.args())
/// });
/// ```
/// 
/// Styles can be re-used to output multiple values:
/// 
/// ```
/// use std::io::Write;
/// use env_logger::fmt::Color;
/// 
/// let mut builder = env_logger::Builder::new();
/// 
/// builder.format(|buf, record| {
///     let mut bold = buf.style();
/// 
///     bold.set_bold(true);
/// 
///     writeln!(buf, "{}: {} {}",
///         bold.value(record.level()),
///         bold.value("some bold text"),
///         record.args())
/// });
/// ```
/// 
/// [`Formatter.style`]: struct.Formatter.html#method.style
/// [color]: #method.color
/// [weight]: #method.weight
/// [`value`]: #method.value
#[derive(Clone)]
pub struct Style {
    buf: Rc<RefCell<Buffer>>,
    write_style: bool,
    spec: ColorSpec,
}

/// A value that can be printed using the given styles.
/// 
/// It is the result of calling [`Style.value`].
/// 
/// [`Style.value`]: struct.Style.html#method.value
pub struct StyledValue<'a, T> {
    style: &'a Style,
    value: T,
}

impl Style {
    /// Set the text color.
    /// 
    /// # Examples
    /// 
    /// Create a style with red text:
    /// 
    /// ```
    /// use std::io::Write;
    /// use env_logger::fmt::Color;
    /// 
    /// let mut builder = env_logger::Builder::new();
    /// 
    /// builder.format(|buf, record| {
    ///     let mut style = buf.style();
    /// 
    ///     style.set_color(Color::Red);
    /// 
    ///     writeln!(buf, "{}", style.value(record.args()))
    /// });
    /// ```
    pub fn set_color(&mut self, color: Color) -> &mut Style {
        self.spec.set_fg(Some(color));
        self
    }

    /// Set the text weight.
    /// 
    /// If `yes` is true then text will be written in bold.
    /// If `yes` is false then text will be written in the default weight.
    /// 
    /// # Examples
    /// 
    /// Create a style with bold text:
    /// 
    /// ```
    /// use std::io::Write;
    /// 
    /// let mut builder = env_logger::Builder::new();
    /// 
    /// builder.format(|buf, record| {
    ///     let mut style = buf.style();
    /// 
    ///     style.set_bold(true);
    /// 
    ///     writeln!(buf, "{}", style.value(record.args()))
    /// });
    /// ```
    pub fn set_bold(&mut self, yes: bool) -> &mut Style {
        self.spec.set_bold(yes);
        self
    }

    /// Set the background color.
    /// 
    /// # Examples
    /// 
    /// Create a style with a yellow background:
    /// 
    /// ```
    /// use std::io::Write;
    /// use env_logger::fmt::Color;
    /// 
    /// let mut builder = env_logger::Builder::new();
    /// 
    /// builder.format(|buf, record| {
    ///     let mut style = buf.style();
    /// 
    ///     style.set_bg(Color::Yellow);
    /// 
    ///     writeln!(buf, "{}", style.value(record.args()))
    /// });
    /// ```
    pub fn set_bg(&mut self, color: Color) -> &mut Style {
        self.spec.set_bg(Some(color));
        self
    }

    /// Wrap a value in the style.
    /// 
    /// The same `Style` can be used to print multiple different values.
    /// 
    /// # Examples
    /// 
    /// Create a bold, red colored style and use it to print the log level:
    /// 
    /// ```
    /// use std::io::Write;
    /// use env_logger::fmt::Color;
    /// 
    /// let mut builder = env_logger::Builder::new();
    /// 
    /// builder.format(|buf, record| {
    ///     let mut style = buf.style();
    /// 
    ///     style.set_color(Color::Red).set_bold(true);
    /// 
    ///     writeln!(buf, "{}: {}",
    ///         style.value(record.level()),
    ///         record.args())
    /// });
    /// ```
    pub fn value<T>(&self, value: T) -> StyledValue<T> {
        StyledValue {
            style: &self,
            value
        }
    }
}

/// An [RFC3339] formatted timestamp.
/// 
/// The timestamp implements [`Display`] and can be written to a [`Formatter`].
/// 
/// [RFC3339]: https://www.ietf.org/rfc/rfc3339.txt
/// [`Display`]: https://doc.rust-lang.org/stable/std/fmt/trait.Display.html
/// [`Formatter`]: struct.Formatter.html
pub struct Timestamp(DateTime<Utc>);

impl Formatter {
    pub(crate) fn new(buf: Buffer, write_style: bool) -> Self {
        Formatter {
            buf: Rc::new(RefCell::new(buf)),
            write_style
        }
    }

    /// Begin a new [`Style`].
    /// 
    /// # Examples
    /// 
    /// Create a bold, red colored style and use it to print the log level:
    /// 
    /// ```
    /// use std::io::Write;
    /// use env_logger::fmt::Color;
    /// 
    /// let mut builder = env_logger::Builder::new();
    /// 
    /// builder.format(|buf, record| {
    ///     let mut level_style = buf.style();
    /// 
    ///     level_style.set_color(Color::Red).set_bold(true);
    /// 
    ///     writeln!(buf, "{}: {}",
    ///         level_style.value(record.level()),
    ///         record.args())
    /// });
    /// ```
    /// 
    /// [`Style`]: struct.Style.html
    pub fn style(&self) -> Style {
        Style {
            buf: self.buf.clone(),
            write_style: self.write_style,
            spec: ColorSpec::new(),
        }
    }

    /// Get a [`Timestamp`] for the current date and time in UTC.
    /// 
    /// # Examples
    /// 
    /// Include the current timestamp with the log record:
    /// 
    /// ```
    /// use std::io::Write;
    /// 
    /// let mut builder = env_logger::Builder::new();
    /// 
    /// builder.format(|buf, record| {
    ///     let ts = buf.timestamp();
    /// 
    ///     writeln!(buf, "{}: {}: {}", ts, record.level(), record.args())
    /// });
    /// ```
    /// 
    /// [`Timestamp`]: struct.Timestamp.html
    pub fn timestamp(&self) -> Timestamp {
        Timestamp(Utc::now())
    }

    pub(crate) fn print(&self, writer: &BufferWriter) -> io::Result<()> {
        writer.print(&self.buf.borrow())
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

impl<'a, T> StyledValue<'a, T> {
    fn write_fmt<F>(&self, f: F) -> fmt::Result
    where
        F: FnOnce() -> fmt::Result,
    {
        if !self.style.write_style {
            // Ignore styles and just run the format function
            return f()
        }

        self.style.buf.borrow_mut().set_color(&self.style.spec).map_err(|_| fmt::Error)?;

        // Always try to reset the terminal style, even if writing failed
        let write = f();
        let reset = self.style.buf.borrow_mut().reset().map_err(|_| fmt::Error);

        write.and(reset)
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /// A `Debug` wrapper for `Timestamp` that uses the `Display` implementation.
        struct TimestampValue<'a>(&'a Timestamp);

        impl<'a> fmt::Debug for TimestampValue<'a> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }

        f.debug_tuple("Timestamp")
         .field(&TimestampValue(&self))
         .finish()
    }
}

impl fmt::Debug for Formatter {
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result {
        f.debug_struct("Formatter").finish()
    }
}

impl fmt::Debug for Style {
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result {
        f.debug_struct("Style").field("spec", &self.spec).finish()
    }
}

macro_rules! impl_styled_value_fmt {
    ($($fmt_trait:path),*) => {
        $(
            impl<'a, T: $fmt_trait> $fmt_trait for StyledValue<'a, T> {
                fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result {
                    self.write_fmt(|| T::fmt(&self.value, f))
                }
            }
        )*
    };
}

impl_styled_value_fmt!(
    fmt::Debug,
    fmt::Display,
    fmt::Pointer,
    fmt::Octal,
    fmt::Binary,
    fmt::UpperHex,
    fmt::LowerHex,
    fmt::UpperExp,
    fmt::LowerExp);

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result {
        const ITEMS: &'static [Item<'static>] = {
            use chrono::format::Item::*;
            use chrono::format::Numeric::*;
            use chrono::format::Fixed::*;
            use chrono::format::Pad::*;

            &[
                Numeric(Year, Zero),
                Literal("-"),
                Numeric(Month, Zero),
                Literal("-"),
                Numeric(Day, Zero),
                Literal("T"),
                Numeric(Hour, Zero),
                Literal(":"),
                Numeric(Minute, Zero),
                Literal(":"),
                Numeric(Second, Zero),
                Fixed(TimezoneOffsetZ),
            ]
        };

        self.0.format_with_items(ITEMS.iter().cloned()).fmt(f)
    }
}
