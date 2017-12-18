/*!
Changing the default logging format.

Before running this example, try setting the `MY_LOG_LEVEL` environment variable to `info`:

```no_run,shell
$ export MY_LOG_LEVEL = 'info'
```

If you want to control the logging output completely, see the `custom_logger` example.
*/

#[macro_use]
extern crate log;
extern crate env_logger;

use std::env;
use std::io::Write;

use env_logger::{Builder, fmt};

fn init_logger() {
    let mut builder = Builder::new();

    // Use a different format for writing log records
    builder.format(|buf, record| {
        let mut style = buf.style();
        style.set_bg(fmt::Color::Yellow).set_bold(true);

        let timestamp = buf.timestamp();

        writeln!(buf, "My formatted log ({}): {}", timestamp, style.value(record.args()))
    });

    if let Ok(s) = env::var("MY_LOG_LEVEL") {
        builder.parse(&s);
    }

    builder.init();
}

fn main() {
    init_logger();

    info!("a log from `MyLogger`");
}
