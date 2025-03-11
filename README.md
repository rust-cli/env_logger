# env_logger

[![crates.io](https://img.shields.io/crates/v/env_logger.svg)](https://crates.io/crates/env_logger)
[![Documentation](https://docs.rs/env_logger/badge.svg)](https://docs.rs/env_logger)

The `env_logger` crate allows you to easily configure and control the logging output of your Rust applications that can be configured using environment variables. This is useful for debugging and monitoring your application without changing the code

## Usage

### In libraries

`env_logger` makes sense when used in executables (binary projects). Libraries should use the [`log`](https://docs.rs/log) crate instead.

### In executables

It must be added along with `log` to the project dependencies:
`log` provides the macros (info!, warn!, error!, debug!, trace!) for logging, while env_logger handles the configuration and filtering of these logs based on environment variables.

```console
$ cargo add log env_logger
```

`env_logger` must be initialized as early as possible in the project. After it's initialized, you can use the `log` macros to do actual logging.

```rust
use log::info;

fn main() {
    env_logger::init();

    info!("starting up");

    // ...
}
```

Then when running the executable, specify a value for the **`RUST_LOG`**
environment variable that corresponds with the log messages you want to show.

```bash
$ RUST_LOG=info ./main
[2018-11-03T06:09:06Z INFO  default] starting up
```
This command sets the logging level to info, meaning that only logs of level info and above (i.e., warn and error) will be displayed.
The letter case is not significant for the logging level names; e.g., `debug`,
`DEBUG`, and `dEbuG` all represent the same logging level. Therefore, the
previous example could also have been written this way, specifying the log
level as `INFO` rather than as `info`:

```bash
$ RUST_LOG=INFO ./main
[2018-11-03T06:09:06Z INFO  default] starting up
```

So which form should you use? For consistency, our convention is to use lower
case names. Where our docs do use other forms, they do so in the context of
specific examples, so you won't be surprised if you see similar usage in the
wild.

#### Module Specific Logging
To set different log levels for different parts of the application.
```bash
$ RUST_LOG=app_module=debug,other_module=warn ./my_app
```
`app_module` will include debug information, while `other_module` will only show warnings and errors. This is useful for focusing on specific areas of your application during development.

### Logging Levels
The log levels that may be specified correspond to the [`log::Level`][level-enum]
enum from the `log` crate. They are:

   * `error` - Logs critical issues that need immediate attention. Use this level for unrecoverable errors or situations where the application must shut down.
   * `warn` - Indicates potential problems that are not immediately harmful but could lead to issues later. Use this level for deprecated features, configuration issues, or performance concerns.
   * `info` - For general operational messages that confirm the application is working as expected. This level is useful for high-level application flow information.
   * `debug` - Provides detailed information that is useful during development and debugging. This level is more verbose than info and can include internal state, variable values, and flow through functions.
   * `trace` - The most granular logging level, used for tracing the program's execution in fine detail. Ideal for troubleshooting complex issues but usually too verbose for general use.

| Level   | Description                                                            |
|---------|------------------------------------------------------------------------|
| `error` | Critical issues needing immediate attention                            |
| `warn`  | Potential problems, not immediately harmful                            |
| `info`  | General information about the application's flow                       |
| `debug` | Detailed information for debugging purposes                            |
| `trace` | Most detailed level, useful for tracing program execution in depth     |



[level-enum]:  https://docs.rs/log/latest/log/enum.Level.html  "log::Level (docs.rs)"

There is also a pseudo logging level, `off`, which may be specified to disable
all logging for a given module or for the entire application. As with the
logging levels, the letter case is not significant.

`env_logger` can be configured in other ways besides an environment variable. See [the examples](https://github.com/rust-cli/env_logger/tree/main/examples) for more approaches.

### In tests

Tests can use the `env_logger` crate to see log messages generated during that test:

```console
$ cargo add log
$ cargo add --dev env_logger
```

```rust
fn add_one(num: i32) -> i32 {
    info!("add_one called with {}", num);
    num + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::info;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn it_adds_one() {
        init();

        info!("can log from the test too");
        assert_eq!(3, add_one(2));
    }

    #[test]
    fn it_handles_negative_numbers() {
        init();

        info!("logging from another test");
        assert_eq!(-7, add_one(-8));
    }
}
```

Assuming the module under test is called `my_lib`, running the tests with the
`RUST_LOG` filtering to info messages from this module looks like:

```bash
$ RUST_LOG=my_lib=info cargo test
     Running target/debug/my_lib-...

running 2 tests
[INFO my_lib::tests] logging from another test
[INFO my_lib] add_one called with -8
test tests::it_handles_negative_numbers ... ok
[INFO my_lib::tests] can log from the test too
[INFO my_lib] add_one called with 2
test tests::it_adds_one ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured
```

Note that `env_logger::try_init()` needs to be called in each test in which you
want to enable logging. Additionally, the default behavior of tests to
run in parallel means that logging output may be interleaved with test output.
Either run tests in a single thread by specifying `RUST_TEST_THREADS=1` or by
running one test by specifying its name as an argument to the test binaries as
directed by the `cargo test` help docs:

```bash
$ RUST_LOG=my_lib=info cargo test it_adds_one
     Running target/debug/my_lib-...

running 1 test
[INFO my_lib::tests] can log from the test too
[INFO my_lib] add_one called with 2
test tests::it_adds_one ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured
```

## Configuring log target

By default, `env_logger` logs to stderr. If you want to log to stdout instead,
you can use the `Builder` to change the log target:

```rust
use std::env;
use env_logger::{Builder, Target};

let mut builder = Builder::from_default_env();
builder.target(Target::Stdout);

builder.init();
```

## Stability of the default format

The default format won't optimise for long-term stability, and explicitly makes no guarantees about the stability of its output across major, minor or patch version bumps during `0.x`.

If you want to capture or interpret the output of `env_logger` programmatically then you should use a custom format.
