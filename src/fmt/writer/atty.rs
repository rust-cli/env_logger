/*
This internal module contains the terminal detection implementation.

If the `auto-color` feature is enabled then we detect whether we're attached to a particular TTY.
Otherwise, assume we're not attached to anything. This effectively prevents styles from being
printed.
*/

use std::io::IsTerminal;

pub(in crate::fmt) fn is_stdout() -> bool {
    cfg!(feature = "auto-color") && std::io::stdout().is_terminal()
}

pub(in crate::fmt) fn is_stderr() -> bool {
    cfg!(feature = "auto-color") && std::io::stderr().is_terminal()
}
