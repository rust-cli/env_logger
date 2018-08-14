#[cfg_attr(feature = "termcolor", path = "extern_impl.rs")]
#[cfg_attr(not(feature = "termcolor"), path = "shim_impl.rs")]
mod imp;

pub(in ::fmt) use self::imp::*;
