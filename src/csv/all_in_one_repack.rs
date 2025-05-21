use std::{
    fs::{self, File},
    io::{self, Read},
};

use atoi::atoi;
use csv::Reader;
use tempfile::TempDir;
use walkdir::WalkDir;

use crate::{Packer, extract::Extractor, mbe_file::MBEFile};

pub fn all_in_one_repack<F: Read>(
    full_text: Reader<F>,
    reference_mvgl: &mut File,
    destination: &mut File,
) -> io::Result<()> {
    let csv_dir = TempDir::new()?;
    super::separate::separate_csv(full_text, csv_dir.path())?;

    let extracted_dir = TempDir::new()?;
    Extractor::new().extract(reference_mvgl, extracted_dir.path())?;

    let translation_dir = TempDir::new()?;
    for file in WalkDir::new(extracted_dir.path()) {
        let file = file?;
        let file_relative_path = file.path().strip_prefix(extracted_dir.path()).unwrap();
        if file.file_type().is_dir() {
            let path = translation_dir.path().join(file_relative_path);
            fs::create_dir_all(path)?;
            continue;
        }
        let csv_path = csv_dir
            .path()
            .join(file_relative_path)
            .with_extension("csv");
        let dest = translation_dir.path().join(file_relative_path);

        let mut source = MBEFile::from_path(&file.path())?;
        source.messages.sort_unstable_by_key(|x| x.message_id);

        if let Ok(reader) = Reader::from_path(csv_path) {
            for entry in reader.into_byte_records() {
                let entry = entry?;
                if let Ok(message) = source
                    .messages
                    .binary_search_by_key(&atoi(&entry[2]).unwrap(), |x| x.message_id)
                {
                    if !entry[0].is_empty() {
                        source.messages[message].text = entry[0].to_vec();
                    }
                }
            }
        }
        let mut dest_file = File::create_new(&dest)?;
        source.write(&mut dest_file)?;
    }

    Packer::new().pack(translation_dir.path(), destination)?;

    Ok(())
}
