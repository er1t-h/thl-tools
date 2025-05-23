mod extract;
mod iterate;
mod pack;

use std::{fmt::Display, ops::Index, path::Path};

pub use extract::Extractor;
pub use iterate::ContentIterator;
pub use pack::Packer;

#[allow(dead_code)]
pub struct FileHeader {
    file_entry_count: u32,
    file_name_count: u32,
    data_entry_count: u32,
    data_start: u64,
    total_size: u64,
}

pub struct FileEntry {
    pub id: u32,
    pub name: String,
}

pub struct FileInfo {
    pub offset: u64,
    pub uncompressed_size: u64,
    pub compressed_size: u64,
    pub associated_struct: FileEntry,
}

#[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord, Clone)]
struct SlicedPath {
    extension: [u8; 4],
    file: String,
}

impl SlicedPath {
    fn new(file: &Path) -> Option<Self> {
        let extension = file.extension()?.to_string_lossy();

        let extension =
            std::array::from_fn(|i| extension.as_bytes().get(i).copied().unwrap_or(b' '));

        Some(Self {
            file: file.with_extension("").to_string_lossy().into_owned(),
            extension,
        })
    }
}

impl Display for SlicedPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}",
            self.file,
            String::from_utf8_lossy(&self.extension).trim_end()
        )
    }
}

impl Index<u16> for SlicedPath {
    type Output = u8;
    fn index(&self, index: u16) -> &Self::Output {
        self.extension
            .iter()
            .chain(self.file.as_bytes().iter())
            .chain(std::iter::once(&0))
            .nth(index as usize)
            .unwrap()
    }
}

const EMPTY_SLICED_PATH: &SlicedPath = &SlicedPath {
    file: String::new(),
    extension: [b' '; 4],
};
