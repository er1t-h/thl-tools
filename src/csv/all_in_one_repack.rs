use std::{
    fs::{self, File},
    io::{self, Read},
};

use csv::Reader;
use tempfile::TempDir;
use walkdir::WalkDir;

use crate::mbe_file::MBEFile;

pub fn all_in_one_repack<F: Read>(
    full_text: Reader<F>,
    reference_mvgl: &mut File,
    destination: &mut File,
) -> io::Result<()> {
    let csv_dir = TempDir::new()?;
    super::separate::separate_csv(full_text, csv_dir.path())?;

    let extracted_dir = TempDir::new()?;
    crate::extract::extract(reference_mvgl, extracted_dir.path())?;

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

        let mut messages = source.messages.iter_mut().peekable();
        if let Ok(reader) = Reader::from_path(csv_path) {
            for entry in reader.into_byte_records() {
                let entry = entry?;
                if let Some(x) =
                    messages.next_if(|x| atoi::atoi(&entry[2]).is_some_and(|i| x.message_id == i))
                {
                    x.text = entry[0].to_vec();
                }
            }
        }
        let mut dest_file = File::create_new(&dest)?;
        source.write(&mut dest_file)?;
    }

    //eprintln!("translation: {}", translation_dir.into_path().display());
    //return Ok(());

    crate::pack(translation_dir.path(), destination)?;

    Ok(())
}
