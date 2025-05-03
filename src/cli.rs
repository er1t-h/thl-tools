use std::{borrow::Cow, path::PathBuf};

use anyhow::{Ok, Result, bail};
use thl_tools::csv::all_in_one_extraction::Language;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Action {
    /// Extracts a `.mvgl` archive to specified folder.
    Extract {
        /// The path to the `.mvgl` archive.
        source: PathBuf,
        /// The path to the folder to create.
        destination: PathBuf,
    },
    /// Packs a folder into a `.mvgl` archive.
    Pack {
        /// The path to the folder.
        source: PathBuf,
        /// The path to the `.mvgl` archive to create.
        destination: PathBuf,
    },
    /// Helps translate the dialogues of a specified `.mbe` file.
    Translate {
        /// The path to the `.mbe` file.
        source: PathBuf,
        /// The path to the resulting `.mbe` file.
        destination: PathBuf,
    },
    /// Read every lines of a `.mbe` file.
    ReadLines {
        /// The path to the `.mbe` file.
        source: PathBuf,
        /// The prefix appended at the beginning of each line.
        #[arg(short, long, default_value_t = Cow::Borrowed("> "))]
        prefix: Cow<'static, str>,
        /// If set to true, will not print twice the same line.
        ///
        /// Particularly useful as some English files seems to have the same line repeated twice.
        #[arg(long, default_value_t = true)]
        ignore_duplicate: bool,
    },
    /// Like translate, but will modify the file instead of creating a new one.
    EditTranslate {
        /// The path to the `.mbe` file.
        source: PathBuf,
    },
    /// Extracts the lines of the `.mbe` file into a CSV file.
    ExtractAsCsv {
        /// The path to the `.mbe` file.
        source: PathBuf,
        /// The path to the `.csv` file created.
        destination: Option<PathBuf>,
    },
    /// Replaces the line from a mbe file with the content of the csv file.
    ReintegrateCsv {
        /// The CSV file to reintegrate.
        csv_file: PathBuf,
        /// The mbe file in which to put the lines. Defaults to `csv_file`, with extension changed
        /// to `.mbe`
        original_mbe_file: Option<PathBuf>,
        /// The optional file in which the new data is stored.
        #[arg(short, long)]
        target: Option<PathBuf>,
    },
    AgglomerateCsv {
        directory: PathBuf,
        destination: PathBuf,
    },
    FuseCsv {
        first_source: PathBuf,
        second_source: PathBuf,
        destination: PathBuf,
    },
    AllInOneExtract {
        game_path: PathBuf,
        #[arg(value_delimiter = ',')]
        languages: Vec<Language>,
    },
    AllInOneRepack {
        full_text: PathBuf,
        reference_mvgl: PathBuf,
        destination: PathBuf,
    },
}

impl Action {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Extract {
                source,
                destination,
            }
            | Self::Translate {
                source,
                destination,
            } => {
                if !source.is_file() {
                    bail!("{} should be a valid file", source.display());
                }
                if destination.exists() && !destination.is_dir() {
                    bail!("{} should not exist", destination.display());
                }
            }
            Self::Pack {
                source,
                destination,
            } => {
                if !source.is_dir() {
                    bail!("{} should be a valid directory", source.display())
                }
                if destination.exists() {
                    bail!("{} should not exist", destination.display())
                }
            }
            Self::EditTranslate { source } | Self::ReadLines { source, .. } => {
                if !source.is_file() {
                    bail!("{} should be a valid file", source.display())
                }
            }
            Self::ExtractAsCsv {
                source,
                destination,
            } => {
                if !source.is_file() {
                    bail!("{} should be a valid file", source.display())
                }
                if let Some(dest) = destination {
                    if dest.exists() {
                        bail!("{} should not exist", dest.display());
                    }
                }
            }
            Self::ReintegrateCsv {
                csv_file,
                original_mbe_file,
                target,
            } => {
                if !csv_file.is_file() {
                    bail!("{} should be a valid file", csv_file.display())
                }
                if let Some(original) = original_mbe_file {
                    if !original.is_file() {
                        bail!("{} should be a valid file", original.display())
                    }
                }
                if let Some(target) = target {
                    if target.exists() {
                        bail!("{} should not exist", target.display())
                    }
                }
            }
            Self::AgglomerateCsv {
                directory,
                destination,
            } => {
                if !directory.is_dir() {
                    bail!("{} should be a valid directory", directory.display());
                }
                if destination.exists() {
                    bail!("{} shouldn't exist", destination.display());
                }
            }
            Self::FuseCsv {
                first_source,
                second_source,
                destination,
            } => {
                if !first_source.is_dir() {
                    bail!("{} should be a valid directory", first_source.display());
                }
                if !second_source.is_dir() {
                    bail!("{} should be a valid directory", second_source.display());
                }
                if destination.exists() {
                    bail!("{} shouldn't exist", destination.display());
                }
            }
            Self::AllInOneExtract {
                game_path,
                languages,
            } => {
                if !game_path.is_dir() {
                    bail!("{} should be a valid directory", game_path.display());
                }
                if languages.is_empty() {
                    bail!("at least one language should be selected");
                }
            }
            Self::AllInOneRepack {
                full_text,
                reference_mvgl,
                destination,
            } => {
                if !full_text.exists() {
                    bail!("{} should exist", full_text.display());
                }
                if !reference_mvgl.exists() {
                    bail!("{} should exist", reference_mvgl.display());
                }
                if destination.exists() {
                    bail!("{} shouldn't exist", destination.display());
                }
            }
        }
        Ok(())
    }
}

///
/// thl-tools: A CLI tool to extract and repack files from the "The Hundred Line" game
///
#[derive(Debug, Clone, clap::Parser)]
pub struct CliArgs {
    /// The subcommand to use:
    ///
    /// - extract
    /// - pack
    /// - translate
    /// - edit-translate
    /// - read-lines
    #[command(subcommand)]
    pub action: Action,
}
