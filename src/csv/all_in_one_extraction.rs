use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::Path,
    time::Duration,
};

use clap::builder::OsStr;
use tempfile::TempDir;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Languages {
    English,
    Japanese,
    SimplifiedChinese,
    TraditionalChinese,
}

impl Languages {
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
            Self::TraditionalChinese => todo!(),
            Self::SimplifiedChinese => todo!(),
        }
    }
}

pub fn all_in_one_extraction(game_path: &Path, languages: &[Languages]) -> io::Result<()> {
    let extraction_dir = TempDir::new()?;

    let languages_dir = std::iter::from_fn(|| Some(TempDir::new()))
        .take(languages.len())
        .collect::<Result<Vec<_>, _>>()?;
    for (&language, extracted_language_dir) in languages.iter().zip(&languages_dir) {
        eprintln!("before extract...");
        crate::extract(
            &mut BufReader::new(File::open(
                game_path.join(format!("gamedata/{}", language.text_file_name())),
            )?),
            extracted_language_dir.path(),
        )?;

        eprintln!("extracting all csv...");
        for file in WalkDir::new(extracted_language_dir) {
            let file = file?;
            if file.file_type().is_dir() {
                eprintln!("creating dir");
                fs::create_dir_all(extraction_dir.path().join(file.path()))?;
                eprintln!("done");
                continue;
            }
            if file.path().extension() == Some(&OsStr::from("csv")) {
                continue;
            }
            let p = extracted_language_dir
                .path()
                .join(file.path().with_extension("csv"));
            eprintln!("creating {}", p.display());
            let destination = File::create_new(p)?;

            let p = extracted_language_dir.path().join(file.path());
            eprintln!("writing from {} to associated csv", p.display());
            crate::csv::extract::extract_as_csv(
                &mut File::open(p)?,
                &destination,
                Some(b"Translated".as_slice()),
                Some(language.as_column_name().as_bytes()),
            )?;
            eprintln!("done");
        }
    }

    let (main, other) = languages_dir.split_first().unwrap();
    eprintln!("fusing all csv...");
    for dir in other {
        let tmp = main.path().with_extension("fuse-tmp");
        crate::csv::fuse::fuse_csv(main.path(), dir.path(), tmp.as_path())?;
        fs::remove_dir_all(main.path())?;
        fs::rename(tmp.as_path(), main.path())?;
    }
    eprintln!("agglomerating csv...");
    crate::csv::agglomerate::agglomerate_csv(main.path(), Path::new("full-text.csv"))?;
    Ok(())
}
