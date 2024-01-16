use super::Filter;
use log::Log;

/// Wrapper that combines a [`Filter`] with an existing [`log::Log`] implementation.
///
/// Records that match the filter will be forwarded to the wrapped log.
/// Other records will be ignored.
#[derive(Debug)]
pub struct FilteredLog<T> {
    filter: Filter,
    log: T,
}

impl<T: Log> FilteredLog<T> {
    /// Create a new filtered log.
    pub fn new(filter: Filter, log: T) -> Self {
        Self { filter, log }
    }

    /// Gets a reference to the filter.
    pub fn filter(&self) -> &Filter {
        &self.filter
    }

    /// Gets a mutable reference to the filter.
    pub fn filter_mut(&mut self) -> &mut Filter {
        &mut self.filter
    }

    /// Gets a reference to the wrapped log.
    pub fn inner(&self) -> &T {
        &self.log
    }

    /// Gets a mutable reference to the wrapped log.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.log
    }

    /// Consumes the filtered log to take back ownership of the filter and the wrapped log.
    pub fn into_parts(self) -> (Filter, T) {
        (self.filter, self.log)
    }
}

impl<T: Log> Log for FilteredLog<T> {
    /// Determines if a log message with the specified metadata would be logged.
    ///
    /// For the wrapped log, this returns `true` only if both the filter and the wrapped log return `true`.
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.filter.enabled(metadata) && self.log.enabled(metadata)
    }

    /// Logs the record.
    ///
    /// Forwards the record to the wrapped log, but only if the record matches the filter.
    fn log(&self, record: &log::Record) {
        if self.filter.matches(record) {
            self.log.log(record)
        }
    }

    /// Flushes any buffered records.
    ///
    /// Forwards directly to the wrapped log.
    fn flush(&self) {
        self.log.flush()
    }
}
