use std::fmt::{self, write};
use std::time::SystemTime;

use chrono::{DateTime, SecondsFormat, Utc};

use crate::fmt::{Formatter, TimestampPrecision};

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
            precision: TimestampPrecision::Seconds,
        }
    }

    /// Get a [`Timestamp`] for the current date and time in UTC with full
    /// second precision.
    pub fn timestamp_seconds(&self) -> Timestamp {
        Timestamp {
            time: Utc::now(),
            precision: TimestampPrecision::Seconds,
        }
    }

    /// Get a [`Timestamp`] for the current date and time in UTC with
    /// millisecond precision.
    pub fn timestamp_millis(&self) -> Timestamp {
        Timestamp {
            time: Utc::now(),
            precision: TimestampPrecision::Millis,
        }
    }

    /// Get a [`Timestamp`] for the current date and time in UTC with
    /// microsecond precision.
    pub fn timestamp_micros(&self) -> Timestamp {
        Timestamp {
            time: Utc::now(),
            precision: TimestampPrecision::Micros,
        }
    }

    /// Get a [`Timestamp`] for the current date and time in UTC with
    /// nanosecond precision.
    pub fn timestamp_nanos(&self) -> Timestamp {
        Timestamp {
            time: Utc::now(),
            precision: TimestampPrecision::Nanos,
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
pub struct Timestamp {
    time: DateTime<Utc>,
    precision: TimestampPrecision,
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
        self.time
            .to_rfc3339_opts(
                match self.precision {
                    TimestampPrecision::Seconds => SecondsFormat::Secs,
                    TimestampPrecision::Millis => SecondsFormat::Millis,
                    TimestampPrecision::Micros => SecondsFormat::Micros,
                    TimestampPrecision::Nanos => SecondsFormat::Nanos,
                },
                true,
            )
            .fmt(f)
    }
}
