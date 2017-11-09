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

use termcolor::{Color, ColorSpec, Buffer, WriteColor};
use chrono::{DateTime, Utc};
use chrono::format::Item;

/// A formatter to write logs into.
/// 
/// `Formatter` implements the standard [`Write`] trait for writing log records.
/// It also supports terminal colors, but this is currently private.
/// 
/// [`Write`]: https://doc.rust-lang.org/stable/std/io/trait.Write.html
pub struct Formatter {
    buf: Buffer,
}

/// A formatter with a particular style.
/// 
/// Each call to `write` will apply the style before writing the output.
pub(crate) struct StyledFormatter<W> {
    buf: W,
    spec: ColorSpec,
}

/// An RFC3339 formatted timestamp.
pub(crate) struct Timestamp(DateTime<Utc>);

impl Formatter {
    pub(crate) fn new(buf: Buffer) -> Self {
        Formatter {
            buf: buf,
        }
    }

    pub(crate) fn color(&mut self, color: Color) -> StyledFormatter<&mut Buffer> {
        let mut spec = ColorSpec::new();
        spec.set_fg(Some(color));

        StyledFormatter {
            buf: &mut self.buf,
            spec: spec
        }
    }

    pub(crate) fn timestamp(&self) -> Timestamp {
        Timestamp(Utc::now())
    }

    pub(crate) fn as_ref(&self) -> &Buffer {
        &self.buf
    }

    pub(crate) fn clear(&mut self) {
        self.buf.clear()
    }
}

impl Write for Formatter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buf.flush()
    }
}

impl<W> Write for StyledFormatter<W>
    where W: WriteColor
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.set_color(&self.spec)?;

        // Always try to reset the terminal style, even if writing failed
        let write = self.buf.write(buf);
        let reset = self.buf.reset();

        write.and_then(|w| reset.map(|_| w))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buf.flush()
    }
}

impl fmt::Debug for Formatter{
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result {
        f.debug_struct("Formatter").finish()
    }
}

impl fmt::Display for Timestamp{
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
