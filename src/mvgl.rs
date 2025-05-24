mod extract;
mod iterate;
mod pack;

use std::{
    fmt::Display,
    fs::File,
    io::{self, BufReader, Read, SeekFrom},
    ops::Index,
    path::Path,
    sync::{Arc, Mutex},
};

use byteorder::{LittleEndian, ReadBytesExt};
pub use extract::Extractor;
pub use iterate::ContentIterator;
use lz4::block::CompressionMode;
pub use pack::Packer;

use crate::helpers::traits::ReadSeek;

pub struct MVGLArchive<R: ReadSeek> {
    header: FileHeader,
    infos: Vec<FileInfo>,
    reader: Arc<Mutex<R>>,
}

impl<R: ReadSeek> MVGLArchive<R> {
    pub fn len(&self) -> usize {
        self.infos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.infos.is_empty()
    }

    fn parse_header(reader: &mut R) -> io::Result<(FileHeader, Vec<FileInfo>)> {
        let mut magic_number = [0; 4];

        reader.read_exact(&mut magic_number)?;
        assert_eq!(&magic_number, b"MDB1");

        let header = FileHeader {
            file_entry_count: reader.read_u32::<LittleEndian>()?,
            file_name_count: reader.read_u32::<LittleEndian>()?,
            data_entry_count: reader.read_u32::<LittleEndian>()?,
            data_start: reader.read_u64::<LittleEndian>()?,
            total_size: reader.read_u64::<LittleEndian>()?,
        };

        let mut sep1 = [0; 16];
        reader.read_exact(&mut sep1)?;
        assert_eq!(
            &sep1,
            [
                0xff_u8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0, 0, 0, 0, 1, 0, 0, 0
            ]
            .as_slice()
        );

        let mut structures = Vec::with_capacity(header.data_entry_count as usize);

        for _ in 0..header.data_entry_count {
            let _compare_byte = reader.read_u32::<LittleEndian>()?;
            let id = reader.read_u32::<LittleEndian>()?;
            let _left = reader.read_u32::<LittleEndian>()?;
            let _right = reader.read_u32::<LittleEndian>()?;
            structures.push(FileEntry {
                id,
                name: String::new(),
            });
        }

        let mut buffer = [0; 0x80];
        reader.read_exact(&mut buffer)?;

        for entry in &mut structures {
            reader.read_exact(&mut buffer)?;
            let extension = buffer[..4]
                .iter()
                .map(|&x| x as char)
                .take_while(|&x| x != ' ');
            let file = buffer[4..]
                .iter()
                .take_while(|&&x| x != 0)
                .map(|&x| x as char)
                .map(|x| if x == '\\' { '/' } else { x })
                .chain(std::iter::once('.'))
                .chain(extension)
                .collect::<String>();
            entry.name = file;
        }

        let mut file_infos = Vec::with_capacity(header.data_entry_count as usize);

        for i in 0..header.data_entry_count {
            let offset = reader.read_u64::<LittleEndian>()?;
            let uncompressed_size = reader.read_u64::<LittleEndian>()?;
            let compressed_size = reader.read_u64::<LittleEndian>()?;

            let position = structures.iter().position(|x| x.id == i).unwrap();
            let structure = structures.swap_remove(position);
            file_infos.push(FileInfo {
                offset,
                decompressed_size: uncompressed_size,
                compressed_size,
                id: structure.id,
                name: structure.name,
            });
        }

        Ok((header, file_infos))
    }

    pub fn from_reader(mut reader: R) -> io::Result<Self> {
        let (header, file_infos) = Self::parse_header(&mut reader)?;
        Ok(Self {
            header,
            infos: file_infos,
            reader: Arc::new(Mutex::new(reader)),
        })
    }

    pub fn get(&self, path: &str) -> Option<io::Result<CompressedFile>> {
        self.infos.iter().find_map(|info| {
            if info.name == path {
                let mut reader = self.reader.lock().unwrap();
                if let Err(e) = reader.seek(SeekFrom::Start(self.header.data_start + info.offset)) {
                    return Some(Err(e));
                };
                Some(CompressedFile::from_reader(
                    reader.by_ref(),
                    info.compressed_size as usize,
                    info.decompressed_size as usize,
                ))
            } else {
                None
            }
        })
    }

    pub fn iter(&self) -> ContentIterator<'_, R> {
        ContentIterator::new(self)
    }
}

impl MVGLArchive<BufReader<File>> {
    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::from_reader(BufReader::new(File::open(path)?))
    }
}

pub struct CompressedFileHandle<'a, R: ReadSeek> {
    info: &'a FileInfo,
    reader: Arc<Mutex<R>>,
    data_start: u64,
}

pub struct CompressedFile {
    content: Vec<u8>,
    decompressed_size: usize,
}
pub struct DecompressedFile {
    content: Vec<u8>,
}

impl<R: ReadSeek> CompressedFileHandle<'_, R> {
    pub fn read(self) -> io::Result<CompressedFile> {
        let mut reader = self.reader.lock().unwrap();
        reader.seek(SeekFrom::Start(self.data_start + self.info.offset))?;

        CompressedFile::from_reader(
            reader.by_ref(),
            self.info.compressed_size as usize,
            self.info.decompressed_size as usize,
        )
    }
}

impl CompressedFile {
    fn from_reader(
        reader: &mut dyn Read,
        compressed_size: usize,
        decompressed_size: usize,
    ) -> io::Result<Self> {
        let mut content = vec![0; compressed_size];
        reader.read_exact(&mut content)?;

        Ok(CompressedFile {
            content,
            decompressed_size,
        })
    }
    pub fn into_inner(self) -> Vec<u8> {
        self.content
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.content
    }

    pub fn decompress(&self) -> Option<DecompressedFile> {
        let decompressed =
            lz4::block::decompress(&self.content, Some(self.decompressed_size as i32)).ok()?;
        Some(DecompressedFile {
            content: decompressed,
        })
    }
}

impl DecompressedFile {
    pub fn into_inner(self) -> Vec<u8> {
        self.content
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.content
    }

    pub fn compress(&self) -> Option<CompressedFile> {
        let compressed = lz4::block::compress(
            &self.content,
            Some(CompressionMode::HIGHCOMPRESSION(12)),
            false,
        )
        .ok()?;
        Some(CompressedFile {
            content: compressed,
            decompressed_size: self.content.len(),
        })
    }
}

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
    pub name: String,
    pub offset: u64,
    pub decompressed_size: u64,
    pub compressed_size: u64,
    pub id: u32,
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
