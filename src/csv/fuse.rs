use std::{collections::HashSet, ffi::OsStr, fs, io, path::Path};

use csv::ByteRecord;
use walkdir::WalkDir;

pub fn fuse_csv(first_source: &Path, second_source: &Path, destination: &Path) -> io::Result<()> {
    let first_source_entries = WalkDir::new(first_source)
        .into_iter()
        .filter_entry(|x| !x.path().starts_with("."))
        .collect::<Result<Vec<_>, _>>()?;

    let second_source_entries = WalkDir::new(second_source)
        .into_iter()
        .filter_entry(|x| !x.path().starts_with("."))
        .collect::<Result<Vec<_>, _>>()?;

    let first_files_path = first_source_entries
        .iter()
        .filter(|x| {
            (x.file_type().is_file() || x.file_type().is_symlink())
                && x.path().extension() == Some(OsStr::new("csv"))
        })
        .flat_map(|x| x.path().strip_prefix(first_source))
        .collect::<HashSet<_>>();

    let second_files_path = second_source_entries
        .iter()
        .filter(|x| {
            (x.file_type().is_file() || x.file_type().is_symlink())
                && x.path().extension() == Some(OsStr::new("csv"))
        })
        .flat_map(|x| x.path().strip_prefix(second_source))
        .collect::<HashSet<_>>();

    for dir in first_source_entries
        .iter()
        .filter(|x| x.file_type().is_dir())
        .map(|x| x.path().strip_prefix(first_source).unwrap())
    {
        let mut dest = destination.to_path_buf();
        dest.push(dir);
        fs::create_dir_all(dest)?;
    }

    let mut byte_record_1 = ByteRecord::new();
    let mut byte_record_2 = ByteRecord::new();

    for path in first_files_path.intersection(&second_files_path) {
        byte_record_1.clear();
        byte_record_2.clear();
        let mut first_source = first_source.to_path_buf();
        let mut second_source = second_source.to_path_buf();
        let mut dest = destination.to_path_buf();

        first_source.push(path);
        second_source.push(path);
        dest.push(path);

        let mut source_2 = csv::Reader::from_path(second_source)?;
        let mut source_1 = csv::Reader::from_path(first_source)?;

        let header = {
            let mut first_header = source_1.byte_headers().unwrap().clone();
            first_header.push_field(&source_2.byte_headers().unwrap()[1]);
            first_header
        };

        let mut destination = csv::Writer::from_path(dest)?;
        destination.write_record(&header)?;

        while [
            source_1.read_byte_record(&mut byte_record_1)?,
            source_2.read_byte_record(&mut byte_record_2)?,
        ]
        .iter()
        .any(|&x| x)
        {
            if byte_record_1.is_empty() {
                byte_record_1.push_field(b"");
                byte_record_1.push_field(b"");
            }
            byte_record_1.push_field(byte_record_2.get(1).unwrap_or(b""));
            destination.write_record(&byte_record_1)?;
        }
    }

    Ok(())
}
