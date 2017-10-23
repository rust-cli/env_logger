//! Filtering for log records.

use std::mem;
use std::fmt;
use log::{Level, LevelFilter, Record, Metadata};

#[cfg(feature = "regex")]
#[path = "regex.rs"]
mod inner;

#[cfg(not(feature = "regex"))]
#[path = "string.rs"]
mod inner;

/// A log filter.
pub struct Filter {
    directives: Vec<Directive>,
    filter: Option<inner::Filter>,
}

#[derive(Debug)]
struct Directive {
    name: Option<String>,
    level: LevelFilter,
}

impl Filter {
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
    /// use env_logger::filter::Builder;
    ///
    /// fn main() {
    ///     let mut builder = Builder::new();
    ///     builder.filter(Some("module1"), LevelFilter::Info);
    ///     builder.filter(Some("module2"), LevelFilter::Error);
    ///
    ///     let filter = builder.build();
    ///     assert_eq!(filter.filter(), LevelFilter::Info);
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
        if !self.enabled(record.metadata()) {
            return false;
        }

        if let Some(filter) = self.filter.as_ref() {
            if !filter.is_match(&*record.args().to_string()) {
                return false;
            }
        }

        true
    }

    /// Check if stuff is enabled.
    pub fn enabled(&self, metadata: &Metadata) -> bool {
        let level = metadata.level();
        let target = metadata.target();

        enabled(&self.directives, level, target)
    }
}

/// A builder for a log filter.
pub struct Builder {
    directives: Vec<Directive>,
    filter: Option<inner::Filter>,
}

impl Builder {
    /// Initializes the log builder with defaults.
    pub fn new() -> Builder {
        Builder {
            directives: Vec::new(),
            filter: None,
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

    /// Build an env logger filter.
    pub fn build(&mut self) -> Filter {
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

        Filter {
            directives: mem::replace(&mut self.directives, Vec::new()),
            filter: mem::replace(&mut self.filter, None),
        }
    }
}

impl fmt::Debug for Filter {
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result {
        f.debug_struct("Filter")
            .field("filter", &self.filter)
            .field("directives", &self.directives)
            .finish()
    }
}

impl fmt::Debug for Builder {
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result {
        f.debug_struct("Filter")
            .field("filter", &self.filter)
            .field("directives", &self.directives)
            .finish()
    }
}

/// Parse a logging specification string (e.g: "crate1,crate2::mod3,crate3::x=error/foo")
/// and return a vector with log directives.
fn parse_spec(spec: &str) -> (Vec<Directive>, Option<inner::Filter>) {
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
        match inner::Filter::new(filter) {
            Ok(re) => Some(re),
            Err(e) => {
                println!("warning: invalid regex filter - {}", e);
                None
            }
        }
    });

    return (dirs, filter);
}


// Check whether a level and target are enabled by the set of directives.
fn enabled(directives: &[Directive], level: Level, target: &str) -> bool {
    // Search for the longest match, the vector is assumed to be pre-sorted.
    for directive in directives.iter().rev() {
        match directive.name {
            Some(ref name) if !target.starts_with(&**name) => {},
            Some(..) | None => {
                return level <= directive.level
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use log::{Level, LevelFilter};

    use super::{Builder, Filter, Directive, parse_spec, enabled};

    fn make_logger_filter(dirs: Vec<Directive>) -> Filter {
        let mut logger = Builder::new().build();
        logger.directives = dirs;
        logger
    }

    #[test]
    fn filter_info() {
        let logger = Builder::new().filter(None, LevelFilter::Info).build();
        assert!(enabled(&logger.directives, Level::Info, "crate1"));
        assert!(!enabled(&logger.directives, Level::Debug, "crate1"));
    }

    #[test]
    fn filter_beginning_longest_match() {
        let logger = Builder::new()
                        .filter(Some("crate2"), LevelFilter::Info)
                        .filter(Some("crate2::mod"), LevelFilter::Debug)
                        .filter(Some("crate1::mod1"), LevelFilter::Warn)
                        .build();
        assert!(enabled(&logger.directives, Level::Debug, "crate2::mod1"));
        assert!(!enabled(&logger.directives, Level::Debug, "crate2"));
    }

    #[test]
    fn parse_default() {
        let logger = Builder::new().parse("info,crate1::mod1=warn").build();
        assert!(enabled(&logger.directives, Level::Warn, "crate1::mod1"));
        assert!(enabled(&logger.directives, Level::Info, "crate2::mod2"));
    }

    #[test]
    fn match_full_path() {
        let logger = make_logger_filter(vec![
            Directive {
                name: Some("crate2".to_string()),
                level: LevelFilter::Info
            },
            Directive {
                name: Some("crate1::mod1".to_string()),
                level: LevelFilter::Warn
            }
        ]);
        assert!(enabled(&logger.directives, Level::Warn, "crate1::mod1"));
        assert!(!enabled(&logger.directives, Level::Info, "crate1::mod1"));
        assert!(enabled(&logger.directives, Level::Info, "crate2"));
        assert!(!enabled(&logger.directives, Level::Debug, "crate2"));
    }

    #[test]
    fn no_match() {
        let logger = make_logger_filter(vec![
            Directive { name: Some("crate2".to_string()), level: LevelFilter::Info },
            Directive { name: Some("crate1::mod1".to_string()), level: LevelFilter::Warn }
        ]);
        assert!(!enabled(&logger.directives, Level::Warn, "crate3"));
    }

    #[test]
    fn match_beginning() {
        let logger = make_logger_filter(vec![
            Directive { name: Some("crate2".to_string()), level: LevelFilter::Info },
            Directive { name: Some("crate1::mod1".to_string()), level: LevelFilter::Warn }
        ]);
        assert!(enabled(&logger.directives, Level::Info, "crate2::mod1"));
    }

    #[test]
    fn match_beginning_longest_match() {
        let logger = make_logger_filter(vec![
            Directive { name: Some("crate2".to_string()), level: LevelFilter::Info },
            Directive { name: Some("crate2::mod".to_string()), level: LevelFilter::Debug },
            Directive { name: Some("crate1::mod1".to_string()), level: LevelFilter::Warn }
        ]);
        assert!(enabled(&logger.directives, Level::Debug, "crate2::mod1"));
        assert!(!enabled(&logger.directives, Level::Debug, "crate2"));
    }

    #[test]
    fn match_default() {
        let logger = make_logger_filter(vec![
            Directive { name: None, level: LevelFilter::Info },
            Directive { name: Some("crate1::mod1".to_string()), level: LevelFilter::Warn }
        ]);
        assert!(enabled(&logger.directives, Level::Warn, "crate1::mod1"));
        assert!(enabled(&logger.directives, Level::Info, "crate2::mod2"));
    }

    #[test]
    fn zero_level() {
        let logger = make_logger_filter(vec![
            Directive { name: None, level: LevelFilter::Info },
            Directive { name: Some("crate1::mod1".to_string()), level: LevelFilter::Off }
        ]);
        assert!(!enabled(&logger.directives, Level::Error, "crate1::mod1"));
        assert!(enabled(&logger.directives, Level::Info, "crate2::mod2"));
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
