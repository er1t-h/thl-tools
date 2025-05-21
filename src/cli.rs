use std::{borrow::Cow, path::PathBuf};

use anyhow::{Ok, Result, bail};
use regex::Regex;
use thl_tools::csv::all_in_one_extraction::Language;

fn get_default_csv_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("full-text.csv");
    path
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Action {
    /// Extracts a `.mvgl` archive to specified folder.
    Extract {
        /// The path to the `.mvgl` archive.
        source: PathBuf,
        /// The path to the folder to create.
        destination: PathBuf,
        /// If true, will not rename `.img` files into `.dds` files.
        #[arg(long)]
        no_rename_images: bool,
        /// Regex that will determine which files to extract.
        #[arg(long)]
        extract_only: Option<Regex>,
    },
    /// Packs a folder into a `.mvgl` archive.
    Pack {
        /// The path to the folder.
        source: PathBuf,
        /// The path to the `.mvgl` archive to create.
        destination: PathBuf,
        /// If true, will overwrite the `destination` file if it exists.
        #[arg(long)]
        overwrite: bool,
        /// If true, will not rename `.dds` files into `.img` files.
        #[arg(long)]
        no_rename_images: bool,
    },
    /// Read every lines of a `.mbe` file.
    ReadLines {
        /// The path to the `.mbe` file.
        source: PathBuf,
        /// The prefix appended at the beginning of each line.
        #[arg(short, long, default_value_t = Cow::Borrowed("> "))]
        prefix: Cow<'static, str>,
    },
    /// Extract all dialogues from the game, putting them all into a single `.csv`
    ExtractDialogues {
        /// The path to the game directory.
        ///
        /// Usually, something like 'C:\Program Files (x86)\Steam\steamapps\common\The Hundred Line -Last Defense Academy-'
        game_path: PathBuf,
        /// The languages that will be exported to the `.csv`, comma separated.
        ///
        /// If you want to export Japanese and English, use 'japanese,english'.
        #[arg(value_delimiter = ',')]
        languages: Vec<Language>,
        /// The path to the destination.
        #[arg(long, default_value=get_default_csv_path().into_os_string())]
        destination: PathBuf,
        /// If true, will overwrite the `destination` file if it exists.
        #[arg(long)]
        overwrite: bool,
    },
    /// Repacks all dialogues in a single `.mvgl` file.
    RepackDialogues {
        /// The path to the `.csv` containing all text.
        full_text: PathBuf,
        /// The `.mvgl` to use to repack.
        ///
        /// The dialogues of the game are stored in GAME_PATH/gamedata/app_text0[LANGUAGE].dx11.mvgl.
        /// Where Japanese is LANGUAGE = 0, English is LANGUAGE = 1, Traditional Chinese is
        /// LANGUAGE = 2 and Simplified Chinese is LANGUAGE = 3
        reference_mvgl: PathBuf,
        /// The path to the repacked text
        destination: PathBuf,
        /// If true, will overwrite the `destination` file if it exists.
        #[arg(long)]
        overwrite: bool,
    },
}

impl CliArgs {
    pub fn validate(&self) -> Result<()> {
        match &self.action {
            Action::Extract {
                source,
                destination,
                no_rename_images: _,
                extract_only: _,
            } => {
                if !source.is_file() {
                    bail!("{} should be a valid file", source.display());
                }
                if destination.exists() && !destination.is_dir() {
                    bail!("{} should not exist", destination.display());
                }
            }
            Action::Pack {
                source,
                destination,
                overwrite,
                no_rename_images: _,
            } => {
                if !source.is_dir() {
                    bail!("{} should be a valid directory", source.display())
                }
                if !*overwrite && destination.exists() {
                    bail!("{} should not exist", destination.display())
                }
            }
            Action::ReadLines { source, .. } => {
                if !source.is_file() {
                    bail!("{} should be a valid file", source.display())
                }
            }
            Action::ExtractDialogues {
                game_path,
                languages,
                destination,
                overwrite,
            } => {
                if !game_path.is_dir() {
                    bail!("{} should be a valid directory", game_path.display());
                }
                if languages.is_empty() {
                    bail!("at least one language should be selected");
                }
                if !*overwrite && destination.exists() {
                    bail!("{} should not exist", destination.display());
                }
            }
            Action::RepackDialogues {
                full_text,
                reference_mvgl,
                destination,
                overwrite,
            } => {
                if !full_text.exists() {
                    bail!("{} should exist", full_text.display());
                }
                if !reference_mvgl.exists() {
                    bail!("{} should exist", reference_mvgl.display());
                }
                if !*overwrite && destination.exists() {
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
    /// The subcommand to use
    #[command(subcommand)]
    pub action: Action,
}
