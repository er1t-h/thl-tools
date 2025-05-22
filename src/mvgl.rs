mod extract;
mod iterate;
mod pack;

pub use extract::Extractor;
pub use iterate::ContentIterator;
pub use pack::Packer;

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
