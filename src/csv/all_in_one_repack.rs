use std::{
    fs::{self, File},
    io::{self, Read},
};

use csv::Reader;
use tempfile::TempDir;
use walkdir::WalkDir;

use crate::translate::{CSVStrategy, NoStrategy};

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
        let dest = translation_dir.path().join(file_relative_path);

        eprintln!("opening {}", file.path().display());
        let mut source = File::open(file.path())?;
        eprintln!("creating {}", dest.display());
        let mut dest = File::create_new(dest)?;
        let mut patcher = crate::translate::Patcher::new(&mut source, &mut dest);
        let csv_path = csv_dir
            .path()
            .join(file_relative_path)
            .with_extension("csv");
        eprintln!("reading from {}", csv_path.display());
        if let Ok(reader) = Reader::from_path(csv_path) {
            patcher.patch(CSVStrategy::new(reader)).unwrap();
        } else {
            patcher.patch(NoStrategy).unwrap();
        }
    }

    crate::pack(translation_dir.path(), destination)?;

    Ok(())
}
