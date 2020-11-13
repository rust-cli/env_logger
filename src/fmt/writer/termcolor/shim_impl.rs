use std::io;

use crate::fmt::{Target, WriteStyle};

pub(in crate::fmt::writer) mod glob {}

pub(in crate::fmt::writer) struct BufferWriter {
    target: Target,
}

pub(in crate::fmt) struct Buffer(Vec<u8>);

impl BufferWriter {
    pub(in crate::fmt::writer) fn stderr(_is_test: bool, _write_style: WriteStyle) -> Self {
        BufferWriter {
            target: Target::Stderr,
        }
    }

    pub(in crate::fmt::writer) fn stdout(_is_test: bool, _write_style: WriteStyle) -> Self {
        BufferWriter {
            target: Target::Stdout,
        }
    }

    pub(in crate::fmt::writer) fn buffer(&self) -> Buffer {
        Buffer(Vec::new())
    }

    pub(in crate::fmt::writer) fn print(&self, buf: &Buffer) -> io::Result<()> {
        use std::io::Write;

        // This impl writes to stdout / stderr instead of using the streams
        // directly.  This is so their output can be captured by `cargo test`
        match self.target {
            Target::Stderr => io::stderr().write_all(&buf.0),
            Target::Stdout => io::stdout().write_all(&buf.0),
        }
        .map(|_| ())
    }
}

impl Buffer {
    pub(in crate::fmt) fn clear(&mut self) {
        self.0.clear();
    }

    pub(in crate::fmt) fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.extend(buf);
        Ok(buf.len())
    }

    pub(in crate::fmt) fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    #[cfg(test)]
    pub(in crate::fmt) fn bytes(&self) -> &[u8] {
        &self.0
    }
}
