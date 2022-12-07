/*
This internal module contains the timestamp implementation.

Its public API is available when the `chrono` crate is available.
*/

#[cfg_attr(feature = "chrono", path = "extern_impl.rs")]
#[cfg_attr(not(feature = "chrono"), path = "shim_impl.rs")]
mod imp;

pub(in crate::fmt) use self::imp::*;
