use std::{
    fs::{self},
    io::{self, Read},
    path::Path,
};

use csv::Writer;
use itertools::Itertools;

pub fn separate_csv<F: Read>(mut source: csv::Reader<F>, destination: &Path) -> io::Result<()> {
    let mut header = source.byte_headers().unwrap().clone();
    let new_header_size = header.len() - 1;
    header.truncate(new_header_size);

    let iter = source
        .byte_records()
        .map(|x| x.unwrap())
        .chunk_by(|x| String::from_utf8(x.get(x.len() - 1).unwrap().to_vec()).unwrap());
    for (file_name, entries) in iter.into_iter() {
        let mut path_to_create = destination.join(Path::new(&file_name));
        let file_path = path_to_create.clone();
        path_to_create.pop();
        fs::create_dir_all(path_to_create)?;

        let mut file = Writer::from_path(file_path)?;
        file.write_record(&header)?;
        for mut entry in entries {
            entry.truncate(new_header_size);
            file.write_record(&entry)?;
        }
    }
    Ok(())
}
