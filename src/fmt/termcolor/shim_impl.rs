use std::io::{self, Write};

use fmt::{WriteStyle, Target};

pub(in ::fmt) mod pub_use_in_super {
    
}

pub(in ::fmt) struct BufferWriter {
    target: Target,
}

pub(in ::fmt) struct Buffer(Vec<u8>);

impl BufferWriter {
    pub(in ::fmt) fn stderr(_: WriteStyle) -> Self {
        BufferWriter {
            target: Target::Stderr,
        }
    }

    pub(in ::fmt) fn stdout(_: WriteStyle) -> Self {
        BufferWriter {
            target: Target::Stdout,
        }
    }

    pub(in ::fmt) fn buffer(&self) -> Buffer {
        Buffer(Vec::new())
    }

    pub(in ::fmt) fn print(&self, buf: &Buffer) -> io::Result<()> {
        match self.target {
            Target::Stderr => io::stderr().write_all(&buf.0),
            Target::Stdout => io::stdout().write_all(&buf.0),
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