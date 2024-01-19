use log::LevelFilter;

use crate::Directive;
use crate::FilterOp;

/// Parse a logging specification string (e.g: "crate1,crate2::mod3,crate3::x=error/foo")
/// and return a vector with log directives.
pub(crate) fn parse_spec(spec: &str) -> (Vec<Directive>, Option<FilterOp>) {
    let mut dirs = Vec::new();

    let mut parts = spec.split('/');
    let mods = parts.next();
    let filter = parts.next();
    if parts.next().is_some() {
        eprintln!(
            "warning: invalid logging spec '{}', \
             ignoring it (too many '/'s)",
            spec
        );
        return (dirs, None);
    }
    if let Some(m) = mods {
        for s in m.split(',').map(|ss| ss.trim()) {
            if s.is_empty() {
                continue;
            }
            let mut parts = s.split('=');
            let (log_level, name) =
                match (parts.next(), parts.next().map(|s| s.trim()), parts.next()) {
                    (Some(part0), None, None) => {
                        // if the single argument is a log-level string or number,
                        // treat that as a global fallback
                        match part0.parse() {
                            Ok(num) => (num, None),
                            Err(_) => (LevelFilter::max(), Some(part0)),
                        }
                    }
                    (Some(part0), Some(""), None) => (LevelFilter::max(), Some(part0)),
                    (Some(part0), Some(part1), None) => match part1.parse() {
                        Ok(num) => (num, Some(part0)),
                        _ => {
                            eprintln!(
                                "warning: invalid logging spec '{}', \
                                 ignoring it",
                                part1
                            );
                            continue;
                        }
                    },
                    _ => {
                        eprintln!(
                            "warning: invalid logging spec '{}', \
                             ignoring it",
                            s
                        );
                        continue;
                    }
                };
            dirs.push(Directive {
                name: name.map(|s| s.to_string()),
                level: log_level,
            });
        }
    }

    let filter = filter.and_then(|filter| match FilterOp::new(filter) {
        Ok(re) => Some(re),
        Err(e) => {
            eprintln!("warning: invalid regex filter - {}", e);
            None
        }
    });

    (dirs, filter)
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;

    use super::parse_spec;

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
    fn parse_spec_empty_level_isolated() {
        // test parse_spec with "" as log level (and the entire spec str)
        let (dirs, filter) = parse_spec(""); // should be ignored
        assert_eq!(dirs.len(), 0);
        assert!(filter.is_none());
    }

    #[test]
    fn parse_spec_blank_level_isolated() {
        // test parse_spec with a white-space-only string specified as the log
        // level (and the entire spec str)
        let (dirs, filter) = parse_spec("     "); // should be ignored
        assert_eq!(dirs.len(), 0);
        assert!(filter.is_none());
    }

    #[test]
    fn parse_spec_blank_level_isolated_comma_only() {
        // The spec should contain zero or more comma-separated string slices,
        // so a comma-only string should be interpreted as two empty strings
        // (which should both be treated as invalid, so ignored).
        let (dirs, filter) = parse_spec(","); // should be ignored
        assert_eq!(dirs.len(), 0);
        assert!(filter.is_none());
    }

    #[test]
    fn parse_spec_blank_level_isolated_comma_blank() {
        // The spec should contain zero or more comma-separated string slices,
        // so this bogus spec should be interpreted as containing one empty
        // string and one blank string. Both should both be treated as
        // invalid, so ignored.
        let (dirs, filter) = parse_spec(",     "); // should be ignored
        assert_eq!(dirs.len(), 0);
        assert!(filter.is_none());
    }

    #[test]
    fn parse_spec_blank_level_isolated_blank_comma() {
        // The spec should contain zero or more comma-separated string slices,
        // so this bogus spec should be interpreted as containing one blank
        // string and one empty string. Both should both be treated as
        // invalid, so ignored.
        let (dirs, filter) = parse_spec("     ,"); // should be ignored
        assert_eq!(dirs.len(), 0);
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
    fn parse_spec_global_bare_warn_lc() {
        // test parse_spec with no crate, in isolation, all lowercase
        let (dirs, filter) = parse_spec("warn");
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].name, None);
        assert_eq!(dirs[0].level, LevelFilter::Warn);
        assert!(filter.is_none());
    }

    #[test]
    fn parse_spec_global_bare_warn_uc() {
        // test parse_spec with no crate, in isolation, all uppercase
        let (dirs, filter) = parse_spec("WARN");
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].name, None);
        assert_eq!(dirs[0].level, LevelFilter::Warn);
        assert!(filter.is_none());
    }

    #[test]
    fn parse_spec_global_bare_warn_mixed() {
        // test parse_spec with no crate, in isolation, mixed case
        let (dirs, filter) = parse_spec("wArN");
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].name, None);
        assert_eq!(dirs[0].level, LevelFilter::Warn);
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
