/*!
Using `env_logger`.

Before running this example, try setting the `MY_LOG_LEVEL` environment variable to `info`:

```no_run,shell
$ export MY_LOG_LEVEL = 'info'
```
*/

#[macro_use]
extern crate log;
extern crate env_logger;

fn main() {
    env_logger::init_from_env("MY_LOG_LEVEL");

    trace!("some trace log");
    debug!("some debug log");
    info!("some information log");
    warn!("some warning log");
    error!("some error log");
}
