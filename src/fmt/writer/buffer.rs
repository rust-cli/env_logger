use std::{io, sync::Mutex};

use crate::fmt::writer::{WritableTarget, WriteStyle};

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

        let buf = buf.as_bytes();
        match &self.target {
            WritableTarget::WriteStdout => {
                let stream = std::io::stdout();
                let mut stream = stream.lock();
                stream.write_all(buf)?;
                stream.flush()?;
            }
            WritableTarget::PrintStdout => print!("{}", String::from_utf8_lossy(buf)),
            WritableTarget::WriteStderr => {
                let stream = std::io::stderr();
                let mut stream = stream.lock();
                stream.write_all(buf)?;
                stream.flush()?;
            }
            WritableTarget::PrintStderr => eprint!("{}", String::from_utf8_lossy(buf)),
            // Safety: If the target type is `Pipe`, `target_pipe` will always be non-empty.
            WritableTarget::Pipe(pipe) => {
                let mut stream = pipe.lock().unwrap();
                stream.write_all(buf)?;
                stream.flush()?;
            }
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

    pub(in crate::fmt) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
