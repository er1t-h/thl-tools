use std::{
    fs::{self, File},
    io::{self, Read},
    time::Duration,
};

use atoi::atoi;
use csv::Reader;
use indicatif::MultiProgress;
use tempfile::TempDir;
use walkdir::WalkDir;

use crate::{
    helpers::traits::{ReadSeekSendSync, WriteSeek},
    mbe::MBEFile,
    mvgl::{Extractor, Packer},
};

///
/// A structure to handle repack of game's dialogues.
///
pub struct DialogueRepacker<'a> {
    multi_progress: Option<&'a MultiProgress>,
}

impl Default for DialogueRepacker<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> DialogueRepacker<'a> {
    pub const fn new() -> Self {
        Self {
            multi_progress: None,
        }
    }

    /// Sets the [`MultiProgress`] to use in the repacking process.
    ///
    /// Useful if you call this function from a script already handling multiple progress bars.
    pub const fn with_multi_progress(self, multi_progress: Option<&'a MultiProgress>) -> Self {
        Self { multi_progress }
    }

    ///
    /// Takes the dialogues from `full_text`, and replaces all matching dialogues in
    /// `reference_mvgl`, writing the result in `destination`
    ///
    pub fn repack(
        &self,
        full_text: &mut dyn Read,
        reference_mvgl: &mut dyn ReadSeekSendSync,
        destination: &mut dyn WriteSeek,
    ) -> io::Result<()> {
        let csv_dir = TempDir::new()?;
        super::separate::separate_csv(Reader::from_reader(full_text), csv_dir.path())?;

        let extracted_dir = TempDir::new()?;
        Extractor::new()
            .with_multi_progress(self.multi_progress)
            .extract(reference_mvgl, extracted_dir.path())?;

        let translation_dir = TempDir::new()?;
        // eprintln!("{}", csv_dir.path().display());
        // std::thread::sleep(Duration::from_secs(60));
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

            //let mut source = MBEFile::from_path(&file.path())?;
            //source.messages.sort_unstable_by_key(|x| x.message_id);
            //
            //if let Ok(reader) = Reader::from_path(&csv_path) {
            //    for (i, entry) in reader.into_byte_records().enumerate() {
            //        let entry = entry?;
            //        if let Ok(message) = source.messages.binary_search_by_key(
            //            &atoi(&entry[3]).unwrap_or_else(|| {
            //                panic!("Row {}, column 3 should be a valid message ID", i)
            //            }),
            //            |x| x.message_id,
            //        ) {
            //            if !entry[0].is_empty() {
            //                source.messages[message].text = entry[0].to_vec();
            //            }
            //        }
            //    }
            //}
            //let mut dest_file = File::create_new(&dest)?;
            //source.write(&mut dest_file)?;
        }

        Packer::new()
            .with_multi_progress(self.multi_progress)
            .pack(translation_dir.path(), destination)?;

        Ok(())
    }
}
