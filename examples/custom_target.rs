/*!
Using `env_logger`.

Before running this example, try setting the `MY_LOG_LEVEL` environment variable to `info`:

```no_run,shell
$ export MY_LOG_LEVEL='info'
```

Also try setting the `MY_LOG_STYLE` environment variable to `never` to disable colors
or `auto` to enable them:

```no_run,shell
$ export MY_LOG_STYLE=never
```
*/

#[macro_use]
extern crate log;

use env_logger::{Builder, Env, Target};
use std::{
    io,
    sync::mpsc::{channel, Sender},
};

// This struct is used as an adaptor, it implements io::Write and forwards the buffer to a mpsc::Sender
struct WriteAdapter {
    sender: Sender<u8>,
}

impl io::Write for WriteAdapter {
    // On write we forward each u8 of the buffer to the sender and return the length of the buffer
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for chr in buf {
            self.sender.send(*chr).unwrap();
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn main() {
    // The `Env` lets us tweak what the environment
    // variables to read are and what the default
    // value is if they're missing
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        // Normally using a pipe as a target would mean a value of false, but this forces it to be true.
        .write_style_or("MY_LOG_STYLE", "always");

    // Create the channel for the log messages
    let (rx, tx) = channel();

    Builder::from_env(env)
        // The Sender of the channel is given to the logger
        // A wrapper is needed, because the `Sender` itself doesn't implement `std::io::Write`.
        .target(Target::Pipe(Box::new(WriteAdapter { sender: rx })))
        .init();

    trace!("some trace log");
    debug!("some debug log");
    info!("some information log");
    warn!("some warning log");
    error!("some error log");

    // Collect all messages send to the channel and parse the result as a string
    String::from_utf8(tx.try_iter().collect::<Vec<u8>>())
        .unwrap()
        // Split the result into lines so a prefix can be added to each line
        .split('\n')
        .for_each(|msg| {
            // Print the message with a prefix if it has any content
            if !msg.is_empty() {
                println!("from pipe: {}", msg)
            }
        });
}
