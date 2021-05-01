mod atty;
mod termcolor;

use self::atty::{is_stderr, is_stdout};
use self::termcolor::BufferWriter;
use std::{
    fmt, io,
    sync::{Arc, Mutex},
};

pub(in crate::fmt) mod glob {
    pub use super::termcolor::glob::*;
    pub use super::*;
}

pub(in crate::fmt) use self::termcolor::Buffer;

/// Log target, either `stdout`, `stderr` or a custom pipe.
#[non_exhaustive]
pub enum Target {
    /// Logs will be sent to standard output.
    Stdout,
    /// Logs will be sent to standard error.
    Stderr,
    /// Logs will be sent to a custom pipe.
    Pipe(Box<dyn io::Write + Send + 'static>),
}

impl Default for Target {
    fn default() -> Self {
        Target::Stderr
    }
}

impl fmt::Debug for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::Stdout => "stdout",
                Self::Stderr => "stderr",
                Self::Pipe(_) => "pipe",
            }
        )
    }
}

/// Log target, either `stdout`, `stderr` or a custom pipe.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub(in crate::fmt) enum TargetType {
    /// Logs will be sent to standard output.
    Stdout,
    /// Logs will be sent to standard error.
    Stderr,
    /// Logs will be sent to a custom pipe.
    Pipe,
}

impl From<&Target> for TargetType {
    fn from(target: &Target) -> Self {
        match target {
            Target::Stdout => Self::Stdout,
            Target::Stderr => Self::Stderr,
            Target::Pipe(_) => Self::Pipe,
        }
    }
}

impl Default for TargetType {
    fn default() -> Self {
        Self::from(&Target::default())
    }
}

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

/// A terminal target with color awareness.
pub(crate) struct Writer {
    inner: BufferWriter,
    write_style: WriteStyle,
}

impl Writer {
    pub fn write_style(&self) -> WriteStyle {
        self.write_style
    }

    pub(in crate::fmt) fn buffer(&self) -> Buffer {
        self.inner.buffer()
    }

    pub(in crate::fmt) fn print(&self, buf: &Buffer) -> io::Result<()> {
        self.inner.print(buf)
    }
}

/// A builder for a terminal writer.
///
/// The target and style choice can be configured before building.
pub(crate) struct Builder {
    target_type: TargetType,
    target_pipe: Option<Arc<Mutex<dyn io::Write + Send + 'static>>>,
    write_style: WriteStyle,
    is_test: bool,
    built: bool,
}

impl Builder {
    /// Initialize the writer builder with defaults.
    pub(crate) fn new() -> Self {
        Builder {
            target_type: Default::default(),
            target_pipe: None,
            write_style: Default::default(),
            is_test: false,
            built: false,
        }
    }

    /// Set the target to write to.
    pub(crate) fn target(&mut self, target: Target) -> &mut Self {
        self.target_type = TargetType::from(&target);
        self.target_pipe = match target {
            Target::Stdout | Target::Stderr => None,
            Target::Pipe(pipe) => Some(Arc::new(Mutex::new(pipe))),
        };
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
                if match self.target_type {
                    TargetType::Stderr => is_stderr(),
                    TargetType::Stdout => is_stdout(),
                    TargetType::Pipe => false,
                } {
                    WriteStyle::Auto
                } else {
                    WriteStyle::Never
                }
            }
            color_choice => color_choice,
        };

        let writer = match self.target_type {
            TargetType::Stderr => BufferWriter::stderr(self.is_test, color_choice),
            TargetType::Stdout => BufferWriter::stdout(self.is_test, color_choice),
            TargetType::Pipe => {
                BufferWriter::pipe(color_choice, self.target_pipe.as_ref().unwrap().clone())
            }
        };

        Writer {
            inner: writer,
            write_style: self.write_style,
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder::new()
    }
}

impl fmt::Debug for Builder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Logger")
            .field("target", &self.target_type)
            .field("write_style", &self.write_style)
            .finish()
    }
}

impl fmt::Debug for Writer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Writer").finish()
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
