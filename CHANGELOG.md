# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate

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
[Unreleased]: https://github.com/rust-cli/env_logger/compare/v0.10.2...HEAD
[0.10.2]: https://github.com/rust-cli/env_logger/compare/v0.10.1...v0.10.2
[0.10.1]: https://github.com/rust-cli/env_logger/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/rust-cli/env_logger/compare/v0.9.3...v0.10.0
[0.9.3]: https://github.com/rust-cli/env_logger/compare/v0.9.2...v0.9.3
[0.9.2]: https://github.com/rust-cli/env_logger/compare/v0.9.0...v0.9.2
[0.9.0]: https://github.com/rust-cli/env_logger/compare/v0.8.4...v0.9.0
