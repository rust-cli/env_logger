/// Log target, either `stdout`, `stderr` or a custom pipe.
#[non_exhaustive]
#[derive(Default)]
pub enum Target {
    /// Logs will be sent to standard output.
    Stdout,
    /// Logs will be sent to standard error.
    #[default]
    Stderr,
    /// Logs will be sent to a custom pipe.
    Pipe(Box<dyn std::io::Write + Send + 'static>),
}



impl std::fmt::Debug for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
///
/// Same as `Target`, except the pipe is wrapped in a mutex for interior mutability.
pub(super) enum WritableTarget {
    /// Logs will be written to standard output.
    WriteStdout,
    /// Logs will be printed to standard output.
    PrintStdout,
    /// Logs will be written to standard error.
    WriteStderr,
    /// Logs will be printed to standard error.
    PrintStderr,
    /// Logs will be sent to a custom pipe.
    Pipe(Box<std::sync::Mutex<dyn std::io::Write + Send + 'static>>),
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
