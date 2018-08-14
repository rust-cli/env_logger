use std::fmt;
use std::io;

use ::WriteStyle;

pub(in ::fmt) mod pub_use {
    
}

pub(in ::fmt) struct BufferWriter {}
pub(in ::fmt) struct Buffer {}

impl BufferWriter {
    pub(in ::fmt) fn stderr(color_choice: WriteStyle) -> Self {
        unimplemented!()
    }

    pub(in ::fmt) fn stdout(color_choice: WriteStyle) -> Self {
        unimplemented!()
    }

    pub(in ::fmt) fn buffer(&self) -> Buffer {
        unimplemented!()
    }

    pub(in ::fmt) fn print(&self, buf: &Buffer) -> io::Result<()> {
        unimplemented!()
    }
}

impl Buffer {
    pub(in ::fmt) fn clear(&mut self) {
        unimplemented!()
    }

    pub(in ::fmt) fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }

    pub(in ::fmt) fn flush(&mut self) -> io::Result<()> {
        unimplemented!()
    }
}