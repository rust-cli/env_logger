use std::fmt;

use chrono::{DateTime, SecondsFormat, Utc};

use crate::fmt::{Formatter, TimestampFormat, TimestampPrecision};

pub(in crate::fmt) mod glob {
    pub use super::*;
}

impl Formatter {
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
        Timestamp {
            time: Utc::now(),
            precision: Default::default(),
            format: Default::default(),
        }
    }

    /// Get a [`Timestamp`] for the current date and time in UTC with a specified style and precision.
    ///
    /// # Examples
    ///
    /// Include the current timestamp, in a 12-hour format to second precision, with the log record:
    ///
    /// ```
    /// use std::io::Write;
    /// use env_logger::fmt;
    ///
    /// let mut builder = env_logger::Builder::new();
    ///
    /// builder.format(|buf, record| {
    ///     let ts = buf.timestamp_custom(fmt::TimestampPrecision::Seconds, fmt::TimestampFormat::Human12Hour);
    ///
    ///     writeln!(buf, "{}: {}: {}", ts, record.level(), record.args())
    /// });
    /// ```
    ///
    /// [`Timestamp`]: struct.Timestamp.html
    pub fn timestamp_custom(
        &self,
        precision: TimestampPrecision,
        format: TimestampFormat,
    ) -> Timestamp {
        Timestamp {
            time: Utc::now(),
            precision,
            format,
        }
    }
}

/// An formatted timestamp.
///
/// The timestamp implements [`Display`] and can be written to a [`Formatter`]. This defaults to formatting with [RFC3339] with second precision.
///
/// [RFC3339]: https://www.ietf.org/rfc/rfc3339.txt
/// [`Display`]: https://doc.rust-lang.org/stable/std/fmt/trait.Display.html
/// [`Formatter`]: struct.Formatter.html
pub struct Timestamp {
    time: DateTime<Utc>,
    precision: TimestampPrecision,
    format: TimestampFormat,
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
            .field(&TimestampValue(self))
            .finish()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.format {
            TimestampFormat::RFC3339 => self
                .time
                .to_rfc3339_opts(
                    match self.precision {
                        TimestampPrecision::Seconds => SecondsFormat::Secs,
                        TimestampPrecision::Millis => SecondsFormat::Millis,
                        TimestampPrecision::Micros => SecondsFormat::Micros,
                        TimestampPrecision::Nanos => SecondsFormat::Nanos,
                    },
                    true,
                )
                .fmt(f),
            TimestampFormat::Human12Hour => {
                if self.precision != TimestampPrecision::Seconds {
                    panic!("Sorry, currently with the new human timestamp formats, we only support second precision.");
                }

                self.time.format("%v %p").fmt(f)
            }
            TimestampFormat::Human24Hour => {
                if self.precision != TimestampPrecision::Seconds {
                    panic!("Sorry, currently with the new human timestamp formats, we only support second precision.");
                }

                self.time.format("%v %X").fmt(f)
            }
        }
    }
}
