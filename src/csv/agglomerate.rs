use std::{
    io::{self, Write},
    path::Path,
};

use csv::ByteRecord;
use walkdir::{DirEntry, WalkDir};

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub fn agglomerate_csv(source: &Path, destination: &mut dyn Write) -> io::Result<()> {
    let mut destination = csv::WriterBuilder::new().from_writer(destination);
    let mut record = ByteRecord::new();
    for (i, file) in WalkDir::new(source)
        .follow_links(true)
        .into_iter()
        .enumerate()
    {
        let file = file?;
        if is_hidden(&file) || !file.file_type().is_file() {
            continue;
        }
        let path = file.path();
        let mut file = csv::ReaderBuilder::new().from_path(path)?;
        if i == 0 {
            file.byte_headers()?.clone_into(&mut record);
            record.push_field(b"file_name");
            destination.write_byte_record(&record)?;
            record.clear();
        }
        let file_name = path.strip_prefix(source).unwrap();
        while file.read_byte_record(&mut record).is_ok_and(|x| x) {
            record.push_field(file_name.to_string_lossy().as_bytes());
            destination.write_byte_record(&record)?;
            record.clear();
        }
    }
    Ok(())
}
