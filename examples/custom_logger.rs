/*!
Using `env_logger` to drive a custom logger.

Before running this example, try setting the `MY_LOG_LEVEL` environment variable to `info`:

```no_run,shell
$ export MY_LOG_LEVEL = 'info'
```

If you only want to change the way logs are formatted, look at the `custom_format` example.
*/

#[macro_use]
extern crate log;
extern crate env_logger;
use env_logger::filter::Filter;
use log::{Log, Metadata, Record, SetLoggerError};

struct MyLogger {
    inner: Filter
}

impl MyLogger {
    fn new() -> MyLogger {
        use env_logger::filter::Builder;
        let mut builder = Builder::new();

        if let Ok(ref filter) = std::env::var("MY_LOG_LEVEL") {
           builder.parse(filter);
        }

        MyLogger {
            inner: builder.build()
        }
    }

    fn init() -> Result<(), SetLoggerError> {
        log::set_boxed_logger(|max_level| {
            let logger = Self::new();
            max_level.set(logger.inner.filter());
            Box::new(logger)
        })
    }
}

impl Log for MyLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if self.inner.matches(record) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) { }
}

fn main() {
    MyLogger::init().unwrap();

    info!("a log from `MyLogger`");
}
