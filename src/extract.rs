use std::{
    collections::HashSet,
    io::{self, Read},
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt};
use indicatif::{MultiProgress, ProgressBar, ProgressIterator};

use crate::indicatif_utils::{IndicatifProgressExt, byte_bar_style_with_message_header};

pub struct Extractor<'a> {
    multi_progress: Option<&'a MultiProgress>,
    rename_images: bool,
}

impl Default for Extractor<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Extractor<'a> {
    pub const fn new() -> Self {
        Self {
            multi_progress: None,
            rename_images: false,
        }
    }

    pub fn with_rename_images(self, rename_images: bool) -> Self {
        Self {
            rename_images,
            ..self
        }
    }

    pub fn with_multi_progress(self, multi_progress: Option<&'a MultiProgress>) -> Self {
        Self {
            multi_progress,
            ..self
        }
    }

    pub fn extract(&self, reader: &mut dyn Read, destination: &Path) -> io::Result<()> {
        struct FileStruct {
            id: u32,
            name: String,
        }

        #[allow(dead_code)]
        struct FileInfo {
            offset: u64,
            uncompressed_size: u64,
            compressed_size: u64,
            content: Vec<u8>,
        }

        std::fs::create_dir_all(destination)?;

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
            structures.push(FileStruct {
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

        let mut file_infos = Vec::new();
        let mut total_compressed_size = 0;

        for _ in 0..data_entry_count {
            let offset = reader.read_u64::<LittleEndian>()?;
            let uncompressed_size = reader.read_u64::<LittleEndian>()?;
            let compressed_size = reader.read_u64::<LittleEndian>()?;
            total_compressed_size += compressed_size;
            file_infos.push(FileInfo {
                offset,
                uncompressed_size,
                compressed_size,
                content: vec![],
            });
        }

        let mut created_dirnames = HashSet::new();
        let progress_bar = ProgressBar::new(total_compressed_size)
            .with_style(byte_bar_style_with_message_header("extracting files"));

        for (i, entry) in file_infos
            .iter_mut()
            .enumerate()
            .progress_with(progress_bar.clone())
            .in_optional_multi_progress(self.multi_progress)
        {
            let mut buffer = vec![0; entry.compressed_size as usize];
            let position = structures.iter().position(|x| x.id == i as u32).unwrap();
            let structure = structures.swap_remove(position);
            progress_bar.set_message(structure.name.clone());

            if let Some((dirname, _)) = structure.name.rsplit_once('/') {
                let format = format!("{}/{}", destination.display(), dirname);
                if !created_dirnames.contains(&format) {
                    std::fs::create_dir_all(&format)?;
                    created_dirnames.insert(format);
                }
            }

            reader.read_exact(&mut buffer)?;

            let res = match lz4::block::decompress(&buffer, Some(entry.uncompressed_size as i32)) {
                Ok(x) => x,
                Err(_) => buffer,
            };

            let file_name = if self.rename_images {
                if let Some(x) = structure.name.strip_suffix(".img") {
                    format!("{}/{x}.dds", destination.display())
                } else {
                    format!("{}/{}", destination.display(), structure.name)
                }
            } else {
                format!("{}/{}", destination.display(), structure.name)
            };

            progress_bar.inc(entry.compressed_size);
            std::fs::write(file_name, &res)?;
        }

        Ok(())
    }
}
