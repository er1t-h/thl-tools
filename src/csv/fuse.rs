use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs, io,
    path::Path,
};

use csv::ByteRecord;
use itertools::Itertools;
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
        let number_of_language_in_src_1 = source_1.byte_headers().unwrap().len() - 3;

        let header = {
            let mut first_header = source_1.byte_headers().unwrap().clone();
            let second_header = source_2.byte_headers().unwrap();
            first_header.push_field(&second_header[3]);
            first_header
        };

        let mut destination = csv::Writer::from_path(dest)?;
        destination.write_record(&header)?;
        let mut byte_record = ByteRecord::new();

        for (left, right) in fuse(
            source_1.byte_records().flatten(),
            source_2.byte_records().flatten(),
            vec![2],
        ) {
            match (left, right) {
                (None, None) => continue,
                (None, Some(right)) => {
                    // Translated
                    byte_record.push_field(b"");
                    // Character
                    byte_record.push_field(right.get(1).unwrap());
                    // Message ID
                    byte_record.push_field(right.get(2).unwrap());
                    // Left Text
                    for _ in 0..number_of_language_in_src_1 {
                        byte_record.push_field(b"");
                    }
                    // Right Text
                    byte_record.push_field(right.get(3).unwrap());
                }
                (Some(left), None) => {
                    // Translated
                    byte_record.push_field(b"");
                    // Character
                    byte_record.push_field(left.get(1).unwrap());
                    // Message ID
                    byte_record.push_field(left.get(2).unwrap());
                    // Left Texts
                    for fields in 3..left.len() {
                        byte_record.push_field(left.get(fields).unwrap());
                    }
                    // Right Text
                    byte_record.push_field(b"");
                }
                (Some(left), Some(right)) => {
                    // Translated
                    byte_record.push_field(b"");
                    // Character
                    byte_record.push_field(left.get(1).unwrap());
                    // Message ID
                    byte_record.push_field(left.get(2).unwrap());
                    // Left Texts
                    for fields in 3..left.len() {
                        byte_record.push_field(left.get(fields).unwrap());
                    }
                    // Right Text
                    byte_record.push_field(right.get(3).unwrap());
                }
            }
            destination.write_byte_record(&byte_record)?;
            byte_record.clear();
        }
    }

    Ok(())
}

fn fuse(
    left: impl Iterator<Item = ByteRecord>,
    right: impl Iterator<Item = ByteRecord>,
    fuse_on: Vec<usize>,
) -> impl Iterator<Item = (Option<ByteRecord>, Option<ByteRecord>)> {
    let mut values = HashMap::new();
    for res in left.zip_longest(right) {
        let (left, right) = res.left_and_right();
        if let Some(left) = left {
            let (l, _) = values
                .entry(
                    fuse_on
                        .iter()
                        .map(|&i| left.get(i).unwrap().to_vec())
                        .collect::<Vec<_>>(),
                )
                .or_default();
            *l = Some(left);
        }

        if let Some(right) = right {
            let (_, r) = values
                .entry(
                    fuse_on
                        .iter()
                        .map(|&i| right.get(i).unwrap().to_vec())
                        .collect(),
                )
                .or_default();
            *r = Some(right);
        }
    }

    values.into_values()
}
