use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, Read},
    iter,
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt};
use indicatif::{ProgressIterator, ProgressStyle};

pub fn extract(reader: &mut dyn Read, destination: &Path) -> io::Result<()> {
    let bar_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {wide_bar} {pos:>7}/{len:7} {msg}")
        .unwrap();

    struct FileStruct {
        id: u32,
        name: String,
    }

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

    for _ in 0..data_entry_count {
        let offset = reader.read_u64::<LittleEndian>()?;
        let uncompressed_size = reader.read_u64::<LittleEndian>()?;
        let compressed_size = reader.read_u64::<LittleEndian>()?;
        file_infos.push(FileInfo {
            offset,
            uncompressed_size,
            compressed_size,
            content: vec![],
        });
    }

    let mut created_dirnames = HashSet::new();

    for (i, entry) in file_infos
        .iter_mut()
        .enumerate()
        .progress_with_style(bar_style)
        .with_message("extracting files")
    {
        let mut buffer = vec![0; entry.compressed_size as usize];
        let position = structures.iter().position(|x| x.id == i as u32).unwrap();
        let structure = structures.swap_remove(position);

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
            Err(e) => {
                eprintln!("error with file {}: {e}", structure.name);

                std::fs::write(
                    format!("{}/{}", destination.display(), structure.name),
                    &buffer,
                )?;
                continue;
            }
        };

        std::fs::write(
            format!("{}/{}", destination.display(), structure.name),
            &res,
        )?;
    }

    Ok(())
}
