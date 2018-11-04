use std::io;

use fmt::{WriteStyle, Target};

pub(in ::fmt::writer) mod pub_use_in_super {
    
}

pub(in ::fmt::writer) struct BufferWriter {
    target: Target,
}

pub(in ::fmt) struct Buffer(Vec<u8>);

impl BufferWriter {
    pub(in ::fmt::writer) fn stderr(_: WriteStyle) -> Self {
        BufferWriter {
            target: Target::Stderr,
        }
    }

    pub(in ::fmt::writer) fn stdout(_: WriteStyle) -> Self {
        BufferWriter {
            target: Target::Stdout,
        }
    }

    pub(in ::fmt::writer) fn buffer(&self) -> Buffer {
        Buffer(Vec::new())
    }

    pub(in ::fmt::writer) fn print(&self, buf: &Buffer) -> io::Result<()> {
        // This impl uses the `eprint` and `print` macros
        // instead of using the streams directly.
        // This is so their output can be captured by `cargo test`
        match self.target {
            Target::Stderr => {
                let log = String::from_utf8_lossy(&buf.0);
                eprint!("{}", log);

                Ok(())
            },
            Target::Stdout => {
                let log = String::from_utf8_lossy(&buf.0);
                print!("{}", log);

                Ok(())
            },
        }
    }
}

impl Buffer {
    pub(in ::fmt) fn clear(&mut self) {
        self.0.clear();
    }

    pub(in ::fmt) fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.extend(buf);
        Ok(buf.len())
    }

    pub(in ::fmt) fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}