use std::{io, sync::Mutex};

use crate::writer::WriteStyle;

#[derive(Debug)]
pub(crate) struct BufferWriter {
    target: WritableTarget,
    write_style: WriteStyle,
}

impl BufferWriter {
    pub(crate) fn stderr(is_test: bool, write_style: WriteStyle) -> Self {
        Self {
            target: if is_test {
                WritableTarget::PrintStderr
            } else {
                WritableTarget::WriteStderr
            },
            write_style,
        }
    }

    pub(crate) fn stdout(is_test: bool, write_style: WriteStyle) -> Self {
        Self {
            target: if is_test {
                WritableTarget::PrintStdout
            } else {
                WritableTarget::WriteStdout
            },
            write_style,
        }
    }

    pub(crate) fn pipe(
        pipe: Box<Mutex<dyn io::Write + Send + 'static>>,
        write_style: WriteStyle,
    ) -> Self {
        Self {
            target: WritableTarget::Pipe(pipe),
            write_style,
        }
    }

    pub(crate) fn write_style(&self) -> WriteStyle {
        self.write_style
    }

    pub(crate) fn buffer(&self) -> Buffer {
        Buffer(Vec::new())
    }

    pub(crate) fn print(&self, buf: &Buffer) -> io::Result<()> {
        #![allow(clippy::print_stdout)] // enabled for tests only
        #![allow(clippy::print_stderr)] // enabled for tests only

        use std::io::Write as _;

        let buf = buf.as_bytes();
        #[cfg(not(feature = "color"))]
        let buf = &escape_controls(buf);
        match &self.target {
            WritableTarget::WriteStdout => {
                let stream = io::stdout();
                #[cfg(feature = "color")]
                let stream = anstream::AutoStream::new(stream, self.write_style.into());
                let mut stream = stream.lock();
                stream.write_all(buf)?;
                stream.flush()?;
            }
            WritableTarget::PrintStdout => {
                #[cfg(feature = "color")]
                let buf = adapt(buf, self.write_style)?;
                #[cfg(feature = "color")]
                let buf = &buf;
                let buf = String::from_utf8_lossy(buf);
                print!("{buf}");
            }
            WritableTarget::WriteStderr => {
                let stream = io::stderr();
                #[cfg(feature = "color")]
                let stream = anstream::AutoStream::new(stream, self.write_style.into());
                let mut stream = stream.lock();
                stream.write_all(buf)?;
                stream.flush()?;
            }
            WritableTarget::PrintStderr => {
                #[cfg(feature = "color")]
                let buf = adapt(buf, self.write_style)?;
                #[cfg(feature = "color")]
                let buf = &buf;
                let buf = String::from_utf8_lossy(buf);
                eprint!("{buf}");
            }
            WritableTarget::Pipe(pipe) => {
                #[cfg(feature = "color")]
                let buf = adapt(buf, self.write_style)?;
                #[cfg(feature = "color")]
                let buf = &buf;
                let mut stream = pipe.lock().expect("no panics while held");
                stream.write_all(buf)?;
                stream.flush()?;
            }
        }

        Ok(())
    }
}

/// Escape control characters as `\xNN`, so that logging untrusted input can't
/// drive the terminal of whoever later reads the output. This build emits no
/// styling of its own, so unlike `anstream` it needs no VT parser to do it.
#[cfg(not(feature = "color"))]
fn escape_controls(buf: &[u8]) -> std::borrow::Cow<'_, [u8]> {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    // Bitwise fold rather than `any`: short-circuiting would force a scalar loop.
    let has_control = buf.iter().fold(false, |acc, &b| acc | is_control(b));
    if !has_control {
        return std::borrow::Cow::Borrowed(buf);
    }

    // Only records that actually carry controls pay for a copy, and the scan
    // above found at least one, each of which grows the record by 3 bytes.
    let mut escaped = Vec::with_capacity(buf.len() + 3);
    for &byte in buf {
        if is_control(byte) {
            let (hi, lo) = (usize::from(byte >> 4), usize::from(byte & 0xf));
            escaped.extend_from_slice(&[b'\\', b'x', HEX[hi], HEX[lo]]);
        } else {
            escaped.push(byte);
        }
    }
    std::borrow::Cow::Owned(escaped)
}

/// Control is all of C0 plus DEL, minus `\n` and `\t`, which the format itself
/// relies on. Stricter than `anstream`, whose strip passes `\r`, VT and FF.
///
/// Multi-byte UTF-8 isn't flagged, having no byte in the tested ASCII range,
/// nor is raw C1; records come from `str`, so only a custom format emits that.
#[cfg(not(feature = "color"))]
#[inline]
fn is_control(byte: u8) -> bool {
    ((byte < 0x20) & (byte != b'\n') & (byte != b'\t')) | (byte == 0x7f)
}

#[cfg(feature = "color")]
fn adapt(buf: &[u8], write_style: WriteStyle) -> io::Result<Vec<u8>> {
    use std::io::Write as _;

    let adapted = Vec::with_capacity(buf.len());
    let mut stream = anstream::AutoStream::new(adapted, write_style.into());
    stream.write_all(buf)?;
    let adapted = stream.into_inner();
    Ok(adapted)
}

pub(crate) struct Buffer(Vec<u8>);

impl Buffer {
    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }

    pub(crate) fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.extend(buf);
        Ok(buf.len())
    }

    pub(crate) fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        String::from_utf8_lossy(self.as_bytes()).fmt(f)
    }
}

/// Log target, either `stdout`, `stderr` or a custom pipe.
///
/// Same as `Target`, except the pipe is wrapped in a mutex for interior mutability.
pub(crate) enum WritableTarget {
    /// Logs will be written to standard output.
    WriteStdout,
    /// Logs will be printed to standard output.
    PrintStdout,
    /// Logs will be written to standard error.
    WriteStderr,
    /// Logs will be printed to standard error.
    PrintStderr,
    /// Logs will be sent to a custom pipe.
    Pipe(Box<Mutex<dyn io::Write + Send + 'static>>),
}

impl std::fmt::Debug for WritableTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::WriteStdout => "stdout",
                Self::PrintStdout => "stdout",
                Self::WriteStderr => "stderr",
                Self::PrintStderr => "stderr",
                Self::Pipe(_) => "pipe",
            }
        )
    }
}
