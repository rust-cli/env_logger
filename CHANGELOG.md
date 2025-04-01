# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate

## [0.11.8] - 2025-04-01

### Compatibility

- *(kv)* Deprecate the `unstable-kv` feature which may be removed in a future patch release

### Features

- *(kv)* Stabilize key-value support behind the `kv` feature
- Expose `ConfigurableFormat` to build custom [`Builder::format`]s that leverage this

## [0.11.7] - 2025-03-10

### Internal

- Replaced `humantime` with `jiff`

## [0.11.6] - 2024-12-20

### Features

- Opt-in file and line rendering

## [0.11.5] - 2024-07-25

## [0.11.4] - 2024-07-23

## [0.11.3] - 2024-03-05

### Features

- Experimental support for key-value logging behind `unstable-kv`

## [0.11.2] - 2024-02-13

## [0.11.1] - 2024-01-27

### Fixes

- Allow styling with `Target::Pipe`

## [0.11.0] - 2024-01-19

### Migration Guide

**env_logger::fmt::Style:**
The bespoke styling API, behind `color`, was removed, in favor of accepting any
ANSI styled string and adapting it to the target stream's capabilities.

Possible styling libraries include:
- [anstyle](https://docs.rs/anstyle) is a minimal, runtime string styling API and is re-exported as `env_logger::fmt::style`
- [owo-colors](https://docs.rs/owo-colors) is a feature rich runtime string styling API
- [color-print](https://docs.rs/color-print) for feature-rich compile-time styling API

[custom_format.rs](https://docs.rs/env_logger/latest/src/custom_format/custom_format.rs.html)
uses `anstyle` via
[`Formatter::default_level_style`](https://docs.rs/env_logger/latest/env_logger/fmt/struct.Formatter.html#method.default_level_style)

### Breaking Change

- Removed bespoke styling API
  - `env_logger::fmt::Formatter::style`
  - `env_logger::fmt::Formatter::default_styled_level`
  - `env_logger::fmt::Style`
  - `env_logger::fmt::Color`
  - `env_logger::fmt::StyledValue`
- Removed `env_logger::filter` in favor of `env_filter`

### Compatibility

MSRV changed to 1.71

### Features

- Automatically adapt ANSI escape codes in logged messages to the current terminal's capabilities
- Add support for `NO_COLOR` and `CLICOLOR_FORCE`, see https://bixense.com/clicolors/

### Fixes

- Print colors when `is_test(true)`

## [0.10.2] - 2024-01-18

### Performance

- Avoid extra UTF-8 validation performed in some cases

### Fixes

- Ensure custom pipes/stdout get flushed
- Don't panic on broken pipes when `color` is disabled

## [0.10.1] - 2023-11-10

### Performance

- Avoid hashing directives and accessing RNG on startup

### Documentation

- Tweak `RUST_LOG` documentation

## [0.10.0] - 2022-11-24

MSRV changed to 1.60 to hide optional dependencies

### Fixes

- Resolved soundness issue by switching from `atty` to `is-terminal`

### Breaking Changes

To open room for changing dependencies:
- Renamed `termcolor` feature to `color`
- Renamed `atty` feature to `auto-color`

## [0.9.3] - 2022-11-07

- Fix a regression from v0.9.2 where env_logger would fail to compile with the termcolor feature turned off.

## [0.9.2] - 2022-11-07

- Fix and un-deprecate Target::Pipe, which was basically not working at all before and deprecated in 0.9.1.

## [0.9.0] -- 2022-07-14

### Breaking Changes

- Default message format now prints the target instead of the module

### Improvements

- Added a method to print the module instead of the target

<!-- next-url -->
[Unreleased]: https://github.com/rust-cli/env_logger/compare/v0.11.8...HEAD
[0.11.8]: https://github.com/rust-cli/env_logger/compare/v0.11.7...v0.11.8
[0.11.7]: https://github.com/rust-cli/env_logger/compare/v0.11.6...v0.11.7
[0.11.6]: https://github.com/rust-cli/env_logger/compare/v0.11.5...v0.11.6
[0.11.5]: https://github.com/rust-cli/env_logger/compare/v0.11.4...v0.11.5
[0.11.4]: https://github.com/rust-cli/env_logger/compare/v0.11.3...v0.11.4
[0.11.3]: https://github.com/rust-cli/env_logger/compare/v0.11.2...v0.11.3
[0.11.2]: https://github.com/rust-cli/env_logger/compare/v0.11.1...v0.11.2
[0.11.1]: https://github.com/rust-cli/env_logger/compare/v0.11.0...v0.11.1
[0.11.0]: https://github.com/rust-cli/env_logger/compare/v0.10.2...v0.11.0
[0.10.2]: https://github.com/rust-cli/env_logger/compare/v0.10.1...v0.10.2
[0.10.1]: https://github.com/rust-cli/env_logger/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/rust-cli/env_logger/compare/v0.9.3...v0.10.0
[0.9.3]: https://github.com/rust-cli/env_logger/compare/v0.9.2...v0.9.3
[0.9.2]: https://github.com/rust-cli/env_logger/compare/v0.9.0...v0.9.2
[0.9.0]: https://github.com/rust-cli/env_logger/compare/v0.8.4...v0.9.0
