use std::io::{self, Read, Write};

pub struct OffsetReadWrapper<'a> {
    offset: usize,
    source: &'a mut dyn Read,
}

impl<'a> OffsetReadWrapper<'a> {
    pub fn new(source: &'a mut dyn Read) -> Self {
        Self { offset: 0, source }
    }
    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl io::Read for OffsetReadWrapper<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = self.source.read(buf)?;
        self.offset += res;
        Ok(res)
    }
}

pub struct OffsetWriteWrapper<'a> {
    offset: usize,
    source: &'a mut dyn Write,
}

impl<'a> OffsetWriteWrapper<'a> {
    pub fn new(source: &'a mut dyn Write) -> Self {
        Self { offset: 0, source }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl io::Write for OffsetWriteWrapper<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = self.source.write(buf)?;
        self.offset += res;
        Ok(res)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.source.flush()
    }
}
