use std::io::{Read, Seek, Write};

pub trait WriteSeek: Write + Seek {}
impl<T: Write + Seek> WriteSeek for T {}

pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}
