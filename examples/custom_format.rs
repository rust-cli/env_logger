/*!
Changing the default logging format.

Before running this example, try setting the `MY_LOG_LEVEL` environment variable to `info`:

```no_run,shell
$ export MY_LOG_LEVEL='info'
```

Also try setting the `MY_LOG_STYLE` environment variable to `never` to disable colors
or `auto` to enable them:

```no_run,shell
$ export MY_LOG_STYLE=never
```

If you want to control the logging output completely, see the `custom_logger` example.
*/

#[cfg(all(feature = "color", feature = "chrono"))]
fn main() {
    use env_logger::{fmt::Color, Builder, Env};

    use std::io::Write;

    fn init_logger() {
        let env = Env::default()
            .filter("MY_LOG_LEVEL")
            .write_style("MY_LOG_STYLE");

        Builder::from_env(env)
            .format(|buf, record| {
                let mut style = buf.style();
                style.set_bg(Color::Yellow).set_bold(true);

                let timestamp = buf.timestamp();

                writeln!(
                    buf,
                    "My formatted log ({}): {}",
                    timestamp,
                    style.value(record.args())
                )
            })
            .init();
    }

    init_logger();

    log::info!("a log from `MyLogger`");
}

#[cfg(not(all(feature = "color", feature = "chrono")))]
fn main() {}
