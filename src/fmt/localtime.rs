use crate::TimestampPrecision;
use chrono::prelude::*;
use std::fmt;
// #[cfg(feature = "localtime")]
pub struct LocalTimestamp {
    datetime: DateTime<Local>,
    precision: TimestampPrecision,
}
// #[cfg(feature = "localtime")]
impl LocalTimestamp {
    /// Get a [`LocalTimestamp`] for the current date and time in UTC with full
    /// second precision.

    pub fn timestamp() -> LocalTimestamp {
        LocalTimestamp {
            datetime: Local::now(),
            precision: TimestampPrecision::Seconds,
        }
    }
    /// Get a [`LocalTimestamp`] for the current date and time in UTC with
    /// millisecond precision.

    pub fn timestamp_seconds() -> LocalTimestamp {
        LocalTimestamp {
            datetime: Local::now(),
            precision: TimestampPrecision::Seconds,
        }
    }
    /// Get a [`LocalTimestamp`] for the current date and time in UTC with
    /// millisecond precision.

    pub fn timestamp_millis() -> LocalTimestamp {
        LocalTimestamp {
            datetime: Local::now(),
            precision: TimestampPrecision::Millis,
        }
    }
    /// Get a [`LocalTimestamp`] for the current date and time in UTC with
    /// microsecond precision.

    pub fn timestamp_micros() -> LocalTimestamp {
        LocalTimestamp {
            datetime: Local::now(),
            precision: TimestampPrecision::Micros,
        }
    }
    /// Get a [`LocalTimestamp`] for the current date and time in UTC with
    /// nanosecond precision.

    pub fn timestamp_nanos() -> LocalTimestamp {
        LocalTimestamp {
            datetime: Local::now(),
            precision: TimestampPrecision::Nanos,
        }
    }
}
// #[cfg(feature = "localtime")]
impl fmt::Display for LocalTimestamp {
    // #[cfg(feature = "localtime")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let formatter = match self.precision {
            TimestampPrecision::Seconds => {
                self.datetime.to_rfc3339_opts(SecondsFormat::Secs, false)
            }
            TimestampPrecision::Millis => {
                self.datetime.to_rfc3339_opts(SecondsFormat::Millis, false)
            }
            TimestampPrecision::Micros => {
                self.datetime.to_rfc3339_opts(SecondsFormat::Micros, false)
            }
            TimestampPrecision::Nanos => self.datetime.to_rfc3339_opts(SecondsFormat::Nanos, false),
        };
        write!(f, "{}", formatter)
    }
}
