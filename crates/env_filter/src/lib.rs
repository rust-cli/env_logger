//! Filtering for log records.
//!
//! You can use the [`Filter`] type in your own logger implementation to use the same
//! filter parsing and matching as `env_logger`.
//!
//! ## Using `env_filter` in your own logger
//!
//! You can use `env_filter`'s filtering functionality with your own logger.
//! Call [`Builder::parse`] to parse directives from a string when constructing
//! your logger. Call [`Filter::matches`] to check whether a record should be
//! logged based on the parsed filters when log records are received.
//!
//! ```
//! use env_filter::Filter;
//! use log::{Log, Metadata, Record};
//!
//! struct MyLogger {
//!     filter: Filter
//! }
//!
//! impl MyLogger {
//!     fn new() -> MyLogger {
//!         use env_filter::Builder;
//!         let mut builder = Builder::new();
//!
//!         // Parse a directives string from an environment variable
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
//!         // Check if the record is matched by the filter
//!         if self.filter.matches(record) {
//!             println!("{:?}", record);
//!         }
//!     }
//!
//!     fn flush(&self) {}
//! }
//! ```

mod directive;
mod filter;
mod op;
mod parser;

use directive::enabled;
use directive::Directive;
use op::FilterOp;
use parser::parse_spec;

pub use filter::Builder;
pub use filter::Filter;
