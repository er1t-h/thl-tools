use std::iter::FusedIterator;

use crate::helpers::traits::ReadSeek;

use super::{CompressedFileHandle, MVGLArchive};

pub struct ContentIterator<'a, R: ReadSeek> {
    pub(super) archive: &'a MVGLArchive<R>,
    index: usize,
}

impl<'a, R: ReadSeek> ContentIterator<'a, R> {
    pub fn new(archive: &'a MVGLArchive<R>) -> Self {
        Self { archive, index: 0 }
    }
}

impl<'a, R: ReadSeek> Iterator for ContentIterator<'a, R> {
    type Item = CompressedFileHandle<'a, R>;

    fn next(&mut self) -> Option<Self::Item> {
        let info = self.archive.infos.get(self.index)?;
        self.index += 1;
        Some(CompressedFileHandle {
            info,
            reader: &self.archive.reader,
            data_start: self.archive.header.data_start,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.archive.infos.len() - self.index;
        (len, Some(len))
    }
}

impl<R: ReadSeek> ExactSizeIterator for ContentIterator<'_, R> {}
impl<R: ReadSeek> FusedIterator for ContentIterator<'_, R> {}
