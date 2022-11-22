/*
This internal module contains the terminal detection implementation.

If the `is-terminal` crate is available then we use it to detect whether
we're attached to a particular TTY. If the `is-terminal` crate is not
available we assume we're not attached to anything. This effectively
prevents styles from being printed.
*/

#[cfg(feature = "is-terminal")]
mod imp {
    use std::io::{stderr, stdout};

    use is_terminal::IsTerminal;

    pub(in crate::fmt) fn is_stdout() -> bool {
        stdout().is_terminal()
    }

    pub(in crate::fmt) fn is_stderr() -> bool {
        stderr().is_terminal()
    }
}

#[cfg(not(feature = "is-terminal"))]
mod imp {
    pub(in crate::fmt) fn is_stdout() -> bool {
        false
    }

    pub(in crate::fmt) fn is_stderr() -> bool {
        false
    }
}

pub(in crate::fmt) use self::imp::*;
