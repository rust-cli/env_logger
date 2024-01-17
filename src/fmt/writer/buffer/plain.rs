use std::{io, sync::Mutex};

use crate::fmt::{WritableTarget, WriteStyle};

pub(in crate::fmt::writer) struct BufferWriter {
    target: WritableTarget,
}

impl BufferWriter {
    pub(in crate::fmt::writer) fn stderr(is_test: bool, _write_style: WriteStyle) -> Self {
        BufferWriter {
            target: if is_test {
                WritableTarget::PrintStderr
            } else {
                WritableTarget::WriteStderr
            },
        }
    }

    pub(in crate::fmt::writer) fn stdout(is_test: bool, _write_style: WriteStyle) -> Self {
        BufferWriter {
            target: if is_test {
                WritableTarget::PrintStdout
            } else {
                WritableTarget::WriteStdout
            },
        }
    }

    pub(in crate::fmt::writer) fn pipe(pipe: Box<Mutex<dyn io::Write + Send + 'static>>) -> Self {
        BufferWriter {
            target: WritableTarget::Pipe(pipe),
        }
    }

    pub(in crate::fmt::writer) fn write_style(&self) -> WriteStyle {
        WriteStyle::Never
    }

    pub(in crate::fmt::writer) fn buffer(&self) -> Buffer {
        Buffer(Vec::new())
    }

    pub(in crate::fmt::writer) fn print(&self, buf: &Buffer) -> io::Result<()> {
        use std::io::Write as _;

        // This impl uses the `eprint` and `print` macros
        // instead of using the streams directly.
        // This is so their output can be captured by `cargo test`.
        match &self.target {
            WritableTarget::WriteStdout => {
                write!(std::io::stdout(), "{}", String::from_utf8_lossy(&buf.0))?
            }
            WritableTarget::PrintStdout => print!("{}", String::from_utf8_lossy(&buf.0)),
            WritableTarget::WriteStderr => {
                write!(std::io::stderr(), "{}", String::from_utf8_lossy(&buf.0))?
            }
            WritableTarget::PrintStderr => eprint!("{}", String::from_utf8_lossy(&buf.0)),
            // Safety: If the target type is `Pipe`, `target_pipe` will always be non-empty.
            WritableTarget::Pipe(pipe) => pipe.lock().unwrap().write_all(&buf.0)?,
        }

        Ok(())
    }
}

pub(in crate::fmt) struct Buffer(Vec<u8>);

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
