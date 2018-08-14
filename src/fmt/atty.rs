#[cfg(feature = "atty")]
mod imp {
    use atty;

    pub(in ::fmt) fn is_stdout() -> bool {
        atty::is(atty::Stream::Stdout)
    }

    pub(in ::fmt) fn is_stderr() -> bool {
        atty::is(atty::Stream::Stderr)
    }
}

#[cfg(not(feature = "atty"))]
mod imp {
    pub(in ::fmt) fn is_stdout() -> bool {
        true
    }

    pub(in ::fmt) fn is_stderr() -> bool {
        true
    }
}

pub(in ::fmt) use self::imp::*;
