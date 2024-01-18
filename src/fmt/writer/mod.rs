mod atty;
mod buffer;
mod target;

use self::atty::{is_stderr, is_stdout};
use self::buffer::BufferWriter;
use std::{fmt, io, mem, sync::Mutex};

pub(super) use self::buffer::Buffer;

pub use target::Target;
use target::WritableTarget;

/// Whether or not to print styles to the target.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WriteStyle {
    /// Try to print styles, but don't force the issue.
    Auto,
    /// Try very hard to print styles.
    Always,
    /// Never print styles.
    Never,
}

impl Default for WriteStyle {
    fn default() -> Self {
        WriteStyle::Auto
    }
}

#[cfg(feature = "color")]
impl WriteStyle {
    fn into_color_choice(self) -> ::termcolor::ColorChoice {
        match self {
            WriteStyle::Always => ::termcolor::ColorChoice::Always,
            WriteStyle::Auto => ::termcolor::ColorChoice::Auto,
            WriteStyle::Never => ::termcolor::ColorChoice::Never,
        }
    }
}

/// A terminal target with color awareness.
pub(crate) struct Writer {
    inner: BufferWriter,
}

impl Writer {
    pub fn write_style(&self) -> WriteStyle {
        self.inner.write_style()
    }

    pub(super) fn buffer(&self) -> Buffer {
        self.inner.buffer()
    }

    pub(super) fn print(&self, buf: &Buffer) -> io::Result<()> {
        self.inner.print(buf)
    }
}

impl fmt::Debug for Writer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Writer").finish()
    }
}

/// A builder for a terminal writer.
///
/// The target and style choice can be configured before building.
#[derive(Debug)]
pub(crate) struct Builder {
    target: Target,
    write_style: WriteStyle,
    is_test: bool,
    built: bool,
}

impl Builder {
    /// Initialize the writer builder with defaults.
    pub(crate) fn new() -> Self {
        Builder {
            target: Default::default(),
            write_style: Default::default(),
            is_test: false,
            built: false,
        }
    }

    /// Set the target to write to.
    pub(crate) fn target(&mut self, target: Target) -> &mut Self {
        self.target = target;
        self
    }

    /// Parses a style choice string.
    ///
    /// See the [Disabling colors] section for more details.
    ///
    /// [Disabling colors]: ../index.html#disabling-colors
    pub(crate) fn parse_write_style(&mut self, write_style: &str) -> &mut Self {
        self.write_style(parse_write_style(write_style))
    }

    /// Whether or not to print style characters when writing.
    pub(crate) fn write_style(&mut self, write_style: WriteStyle) -> &mut Self {
        self.write_style = write_style;
        self
    }

    /// Whether or not to capture logs for `cargo test`.
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn is_test(&mut self, is_test: bool) -> &mut Self {
        self.is_test = is_test;
        self
    }

    /// Build a terminal writer.
    pub(crate) fn build(&mut self) -> Writer {
        assert!(!self.built, "attempt to re-use consumed builder");
        self.built = true;

        let color_choice = match self.write_style {
            WriteStyle::Auto => {
                if match &self.target {
                    Target::Stderr => is_stderr(),
                    Target::Stdout => is_stdout(),
                    Target::Pipe(_) => false,
                } {
                    WriteStyle::Auto
                } else {
                    WriteStyle::Never
                }
            }
            color_choice => color_choice,
        };
        let color_choice = if self.is_test {
            WriteStyle::Never
        } else {
            color_choice
        };

        let writer = match mem::take(&mut self.target) {
            Target::Stderr => BufferWriter::stderr(self.is_test, color_choice),
            Target::Stdout => BufferWriter::stdout(self.is_test, color_choice),
            Target::Pipe(pipe) => BufferWriter::pipe(Box::new(Mutex::new(pipe))),
        };

        Writer { inner: writer }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder::new()
    }
}

fn parse_write_style(spec: &str) -> WriteStyle {
    match spec {
        "auto" => WriteStyle::Auto,
        "always" => WriteStyle::Always,
        "never" => WriteStyle::Never,
        _ => Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_write_style_valid() {
        let inputs = vec![
            ("auto", WriteStyle::Auto),
            ("always", WriteStyle::Always),
            ("never", WriteStyle::Never),
        ];

        for (input, expected) in inputs {
            assert_eq!(expected, parse_write_style(input));
        }
    }

    #[test]
    fn parse_write_style_invalid() {
        let inputs = vec!["", "true", "false", "NEVER!!"];

        for input in inputs {
            assert_eq!(WriteStyle::Auto, parse_write_style(input));
        }
    }
}
