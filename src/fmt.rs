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
/// [`Write`]: https://doc.rust-lang.org/stable/std/io/trait.Write.html
/// [`style`]: #method.style
pub struct Formatter {
    buf: Rc<RefCell<Buffer>>,
}

/// A set of styles to apply to the terminal output.
#[derive(Clone)]
pub struct Style {
    buf: Rc<RefCell<Buffer>>,
    spec: ColorSpec,
}

/// A value that can be printed using the given styles.
pub struct StyledValue<'a, T> {
    style: &'a Style,
    value: T,
}

impl Style {
    /// Set the foreground color.
    pub fn set_color(&mut self, color: Color) -> &mut Style {
        self.spec.set_fg(Some(color));
        self
    }

    /// Make the text bold.
    pub fn set_bold(&mut self, yes: bool) -> &mut Style {
        self.spec.set_bold(yes);
        self
    }

    /// Set the background color.
    pub fn set_bg(&mut self, color: Color) -> &mut Style {
        self.spec.set_bg(Some(color));
        self
    }

    /// Wrap a value in the style.
    /// 
    /// The same `Style` can be used to print multiple different values.
    pub fn value<T>(&self, value: T) -> StyledValue<T> {
        StyledValue {
            style: &self,
            value
        }
    }
}

/// An RFC3339 formatted timestamp.
/// 
/// The timestamp implements [`Display`] and can be written to a [`Formatter`].
/// 
/// [`Display`]: https://doc.rust-lang.org/stable/std/fmt/trait.Display.html
/// [`Formatter`]: struct.Formatter.html
pub struct Timestamp(DateTime<Utc>);

impl Formatter {
    pub(crate) fn new(buf: Buffer) -> Self {
        Formatter {
            buf: Rc::new(RefCell::new(buf)),
        }
    }

    /// Begin a new style.
    pub fn style(&self) -> Style {
        Style {
            buf: self.buf.clone(),
            spec: ColorSpec::new(),
        }
    }

    /// Get a timestamp.
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

impl<'a, T: fmt::Debug> fmt::Debug for StyledValue<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result {
        self.write_fmt(|| T::fmt(&self.value, f))
    }
}

impl<'a, T: fmt::Display> fmt::Display for StyledValue<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result {
        self.write_fmt(|| T::fmt(&self.value, f))
    }
}

// TODO: Other `fmt` traits

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
