use std::{
    fs::{self, File},
    io::{self, BufWriter, Read},
};

use atoi::atoi;
use byte_string::ByteStr;
use csv::Reader;
use indicatif::MultiProgress;
use tempfile::TempDir;
use walkdir::WalkDir;

use crate::{
    helpers::{
        offset_wrapper::OffsetWriteWrapper,
        traits::{ReadSeekSendSync, WriteSeek},
    },
    mbe::{ColumnType, MBEFile, TableCell},
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

            let mut source = MBEFile::from_path(file.path()).unwrap();

            if let Ok(reader) = Reader::from_path(&csv_path) {
                for entry in reader.into_byte_records() {
                    let entry = entry?;
                    let mut rows = source.rows();
                    if rows.by_ref().any(|x| match x[0] {
                        TableCell::Int(x) | TableCell::IntID(x) => x == atoi(&entry[0]).unwrap(),
                        TableCell::StringID(Some(x)) | TableCell::String(Some(x)) => x == &entry[0],
                        _ => panic!(),
                    }) && !entry[2].is_empty()
                    {
                        let (sheet, row) = if rows.row() == 0 {
                            let x = source.get_sheet_by_index(rows.sheet() - 1).unwrap();
                            (rows.sheet() - 1, x.number_of_row().saturating_sub(1))
                        } else {
                            (rows.sheet(), rows.row() - 1)
                        };
                        let column = if let Some(sheet) = source.get_sheet_by_index(sheet)
                            && let Some(content) = sheet
                                .column_types()
                                .iter()
                                .position(|&x| x == ColumnType::String)
                        {
                            content
                        } else {
                            1
                        };
                        if csv_path.ends_with("help_tutorial_text.csv") {
                            println!(
                                "{:?}",
                                source.get_sheet_by_index(sheet).unwrap().column_types()
                            );
                            println!(
                                "modifying string from sheet{sheet}, row{row} and column{column} to {:?}",
                                ByteStr::new(&entry[2])
                            );
                            println!(
                                "{:?}",
                                source.modify_string(sheet, row, column, entry[2].to_vec())
                            );
                        } else {
                            source.modify_string(sheet, row, column, entry[2].to_vec());
                        }
                    }
                }
            }
            let mut dest_file = BufWriter::new(File::create_new(&dest)?);
            source.write(&mut OffsetWriteWrapper::new(&mut dest_file))?;
        }

        Packer::new()
            .with_multi_progress(self.multi_progress)
            .pack(translation_dir.path(), destination)?;

        Ok(())
    }
}
