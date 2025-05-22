use std::{
    collections::VecDeque,
    io::{self, Read},
    iter::FusedIterator,
};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{helpers::traits::ReadSeek, mvgl::FileEntry};

use super::FileInfo;

pub struct ContentIterator<'a> {
    file_infos: VecDeque<FileInfo>,
    reader: &'a mut dyn ReadSeek,
    should_decompress: bool,
}

impl<'a> ContentIterator<'a> {
    fn parse_header(reader: &mut dyn ReadSeek) -> io::Result<VecDeque<FileInfo>> {
        let mut magic_number = [0; 4];

        reader.read_exact(&mut magic_number)?;
        assert_eq!(&magic_number, b"MDB1");

        let _file_entry_count = reader.read_u32::<LittleEndian>()?;
        let _file_name_count = reader.read_u32::<LittleEndian>()?;
        let data_entry_count = reader.read_u32::<LittleEndian>()?;
        let _data_start = reader.read_u64::<LittleEndian>()?;
        let _total_size = reader.read_u64::<LittleEndian>()?;

        let mut sep1 = [0; 16];
        reader.read_exact(&mut sep1)?;
        assert_eq!(
            &sep1,
            [
                0xff_u8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0, 0, 0, 0, 1, 0, 0, 0
            ]
            .as_slice()
        );

        let mut structures = Vec::with_capacity(data_entry_count as usize);

        for _ in 0..data_entry_count {
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

        let mut file_infos = VecDeque::with_capacity(data_entry_count as usize);

        for i in 0..data_entry_count {
            let offset = reader.read_u64::<LittleEndian>()?;
            let uncompressed_size = reader.read_u64::<LittleEndian>()?;
            let compressed_size = reader.read_u64::<LittleEndian>()?;

            let position = structures.iter().position(|x| x.id == i).unwrap();
            let structure = structures.swap_remove(position);
            file_infos.push_back(FileInfo {
                offset,
                uncompressed_size,
                compressed_size,
                associated_struct: structure,
            });
        }

        Ok(file_infos)
    }

    pub fn new(reader: &'a mut dyn ReadSeek) -> io::Result<Self> {
        Ok(Self {
            file_infos: Self::parse_header(reader)?,
            reader,
            should_decompress: true,
        })
    }

    pub fn with_should_decompress(mut self, should_decompress: bool) -> Self {
        self.set_should_decompress(should_decompress);
        self
    }

    pub fn set_should_decompress(&mut self, should_decompress: bool) {
        self.should_decompress = should_decompress;
    }

    pub fn file_infos(&self) -> &VecDeque<FileInfo> {
        &self.file_infos
    }
}

impl Iterator for ContentIterator<'_> {
    type Item = io::Result<(FileInfo, Vec<u8>)>;
    fn next(&mut self) -> Option<Self::Item> {
        let file_info = self.file_infos.pop_front()?;
        let mut content = Vec::new();
        if let Err(e) = self
            .reader
            .take(file_info.compressed_size)
            .read_to_end(&mut content)
        {
            return Some(Err(e));
        };
        let content = if self.should_decompress {
            match lz4::block::decompress(&content, file_info.uncompressed_size.try_into().ok()) {
                Ok(x) => x,
                Err(e) => return Some(Err(e)),
            }
        } else {
            content
        };
        Some(Ok((file_info, content)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.file_infos.iter().size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        for _ in 0..n {
            let file_info = self.file_infos.pop_front()?;
            if let Err(e) = self.reader.seek_relative(file_info.compressed_size as i64) {
                return Some(Err(e));
            }
        }
        self.next()
    }
}

impl ExactSizeIterator for ContentIterator<'_> {}
impl FusedIterator for ContentIterator<'_> {}
