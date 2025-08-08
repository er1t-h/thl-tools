use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufReader, Write},
    path::Path,
    time::Duration,
};

use csv::Writer;
use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressIterator};
use tempfile::TempDir;
use walkdir::WalkDir;

use crate::{
    helpers::indicatif::{
        IndicatifProgressExt, default_bar_style_with_message_header, default_spinner_style,
        default_spinner_style_with_message_header,
    },
    mvgl::Extractor,
};

pub struct DialogueExtractor<'a> {
    multi_progress: Option<&'a MultiProgress>,
}

impl Default for DialogueExtractor<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> DialogueExtractor<'a> {
    pub const fn new() -> Self {
        Self {
            multi_progress: None,
        }
    }

    pub const fn with_multi_progress(multi_progress: Option<&'a MultiProgress>) -> Self {
        Self { multi_progress }
    }

    pub fn extract(
        &self,
        languages: &[(impl AsRef<Path>, impl AsRef<str>)],
        destination: &mut dyn Write,
    ) -> io::Result<()> {
        let extraction_dir = TempDir::new()?;

        let languages_dir = std::iter::from_fn(|| Some(TempDir::new()))
            .take(languages.len())
            .collect::<Result<Vec<_>, _>>()?;

        let multi_progress = self
            .multi_progress
            .map_or_else(|| Cow::Owned(MultiProgress::default()), Cow::Borrowed);

        let progress_bar = ProgressBar::new(languages.len() as u64)
            .with_style(default_bar_style_with_message_header("working on language"));

        for ((lang_path, lang_name), extracted_language_dir) in languages
            .iter()
            .zip(&languages_dir)
            .progress_with(progress_bar.clone())
            .in_multi_progress(&multi_progress)
            .with_finish(ProgressFinish::WithMessage(
                "finished extracting all files".into(),
            ))
        {
            progress_bar.set_message(lang_name.as_ref().to_string());
            Extractor::new()
                .with_multi_progress(Some(&multi_progress))
                .extract(
                    &mut BufReader::new(File::open(lang_path)?),
                    extracted_language_dir.path(),
                )?;

            let spinner = ProgressBar::new_spinner().with_style(
                default_spinner_style_with_message_header("creating individual CSV for"),
            );
            multi_progress.add(spinner.clone());
            spinner.enable_steady_tick(Duration::from_millis(200));

            for file in WalkDir::new(extracted_language_dir) {
                let file = file?;
                //if (!file
                //    .path()
                //    .ends_with("s15_a_deathgam_aaaab_d078_m020_010.mbe"))
                //{
                //    continue;
                //}
                if file.file_type().is_dir() {
                    fs::create_dir_all(extraction_dir.path().join(file.path()))?;
                    continue;
                }
                if file.path().extension() != Some(OsStr::new("mbe")) {
                    continue;
                }

                spinner.set_message(
                    file.path()
                        .strip_prefix(extracted_language_dir.path())
                        .unwrap()
                        .display()
                        .to_string(),
                );

                let p = extracted_language_dir
                    .path()
                    .join(file.path().with_extension("csv"));
                let mut destination = File::create_new(p)?;

                let p = extracted_language_dir.path().join(file.path());
                crate::csv::extract::extract_as_csv(
                    &mut BufReader::new(File::open(p)?),
                    &mut Writer::from_writer(&mut destination),
                    Some(b"Translated".as_slice()),
                    Some(lang_name.as_ref().as_bytes()),
                )?;
            }
        }

        let (main, other) = languages_dir.split_first().unwrap();

        let progress_bar = ProgressBar::new_spinner()
            .with_style(default_spinner_style())
            .with_message("fusing CSVs");
        progress_bar.enable_steady_tick(Duration::from_millis(200));
        for dir in other {
            let tmp = main.path().with_extension("fuse-tmp");
            crate::csv::fuse::fuse_csv(main.path(), dir.path(), tmp.as_path())?;
            fs::remove_dir_all(main.path())?;
            fs::rename(tmp.as_path(), main.path())?;
        }
        progress_bar.finish_and_clear();

        let progress_bar = ProgressBar::new_spinner()
            .with_style(default_spinner_style())
            .with_message("agglomerating CSVs");
        progress_bar.enable_steady_tick(Duration::from_millis(200));
        eprintln!("aglomerating");
        crate::csv::agglomerate::agglomerate_csv(main.path(), destination)?;
        progress_bar.finish_and_clear();

        Ok(())
    }
}
