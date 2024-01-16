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
    #[allow(dead_code)]
    WriteStdout,
    /// Logs will be printed to standard output.
    PrintStdout,
    /// Logs will be written to standard error.
    #[allow(dead_code)]
    WriteStderr,
    /// Logs will be printed to standard error.
    PrintStderr,
    /// Logs will be sent to a custom pipe.
    Pipe(Box<std::sync::Mutex<dyn std::io::Write + Send + 'static>>),
}

impl WritableTarget {
    pub(super) fn print(&self, buf: &super::Buffer) -> std::io::Result<()> {
        use std::io::Write as _;

        let buf = buf.as_bytes();
        match self {
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
