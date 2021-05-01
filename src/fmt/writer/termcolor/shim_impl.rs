use std::{
    io,
    sync::{Arc, Mutex},
};

use crate::fmt::{TargetType, WriteStyle};

pub(in crate::fmt::writer) mod glob {}

pub(in crate::fmt::writer) struct BufferWriter {
    target: TargetType,
    target_pipe: Option<Box<Mutex<dyn io::Write + Send + 'static>>>,
}

pub(in crate::fmt) struct Buffer(Vec<u8>);

impl BufferWriter {
    pub(in crate::fmt::writer) fn stderr(_is_test: bool, _write_style: WriteStyle) -> Self {
        BufferWriter {
            target: TargetType::Stderr,
            target_pipe: None,
        }
    }

    pub(in crate::fmt::writer) fn stdout(_is_test: bool, _write_style: WriteStyle) -> Self {
        BufferWriter {
            target: TargetType::Stdout,
            target_pipe: None,
        }
    }

    pub(in crate::fmt::writer) fn pipe(
        _write_style: WriteStyle,
        target_pipe: Box<Mutex<dyn io::Write + Send + 'static>>,
    ) -> Self {
        BufferWriter {
            target: TargetType::Pipe,
            target_pipe: Some(target_pipe),
        }
    }

    pub(in crate::fmt::writer) fn buffer(&self) -> Buffer {
        Buffer(Vec::new())
    }

    pub(in crate::fmt::writer) fn print(&self, buf: &Buffer) -> io::Result<()> {
        // This impl uses the `eprint` and `print` macros
        // instead of using the streams directly.
        // This is so their output can be captured by `cargo test`.
        match self.target {
            // Safety: If the target type is `Pipe`, `target_pipe` will always be non-empty.
            TargetType::Pipe => self
                .target_pipe
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .write_all(&buf.0)?,
            TargetType::Stdout => print!("{}", String::from_utf8_lossy(&buf.0)),
            TargetType::Stderr => eprint!("{}", String::from_utf8_lossy(&buf.0)),
        }

        Ok(())
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
