use std::{
    fs::{self, File},
    io::{self, BufReader, Write},
    path::Path,
    time::Duration,
};

use clap::builder::OsStr;
use indicatif::{MultiProgress, ProgressBar, ProgressIterator};
use tempfile::TempDir;
use walkdir::WalkDir;

use crate::{
    Extractor,
    indicatif_utils::{
        IndicatifProgressExt, default_bar_style_with_message_header, default_spinner_style,
        default_spinner_style_with_message_header,
    },
};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Language {
    English,
    Japanese,
    SimplifiedChinese,
    TraditionalChinese,
}

impl Language {
    fn as_column_name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Japanese => "Japanese",
            Self::TraditionalChinese => "Traditional Chinese",
            Self::SimplifiedChinese => "Simplified Chinese",
        }
    }

    fn text_file_name(self) -> &'static str {
        match self {
            Self::Japanese => "app_text00.dx11.mvgl",
            Self::English => "app_text01.dx11.mvgl",
            Self::TraditionalChinese => "app_text02.dx11.mvgl",
            Self::SimplifiedChinese => "app_text03.dx11.mvgl",
        }
    }
}

pub fn all_in_one_extraction(
    game_path: &Path,
    languages: &[Language],
    destination: &mut dyn Write,
) -> io::Result<()> {
    let extraction_dir = TempDir::new()?;

    let languages_dir = std::iter::from_fn(|| Some(TempDir::new()))
        .take(languages.len())
        .collect::<Result<Vec<_>, _>>()?;
    let multi_progress = MultiProgress::new();

    let progress_bar = ProgressBar::new(languages.len() as u64)
        .with_style(default_bar_style_with_message_header("working on language"));

    for (&language, extracted_language_dir) in languages
        .iter()
        .zip(&languages_dir)
        .progress_with(progress_bar.clone())
        .in_multi_progress(&multi_progress)
    {
        progress_bar.set_message(language.as_column_name());
        Extractor::new()
            .with_multi_progress(Some(&multi_progress))
            .extract(
                &mut BufReader::new(File::open(
                    game_path.join(format!("gamedata/{}", language.text_file_name())),
                )?),
                extracted_language_dir.path(),
            )?;

        let spinner = ProgressBar::new_spinner().with_style(
            default_spinner_style_with_message_header("creating individual CSV for"),
        );
        multi_progress.add(spinner.clone());
        spinner.enable_steady_tick(Duration::from_millis(200));

        for file in WalkDir::new(extracted_language_dir) {
            let file = file?;
            if file.file_type().is_dir() {
                fs::create_dir_all(extraction_dir.path().join(file.path()))?;
                continue;
            }
            if file.path().extension() != Some(&OsStr::from("mbe")) {
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
            let destination = File::create_new(p)?;

            let p = extracted_language_dir.path().join(file.path());
            crate::csv::extract::extract_as_csv(
                &mut File::open(p)?,
                &destination,
                Some(b"Translated".as_slice()),
                Some(language.as_column_name().as_bytes()),
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
    crate::csv::agglomerate::agglomerate_csv(main.path(), destination)?;
    progress_bar.finish_and_clear();

    Ok(())
}
