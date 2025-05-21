use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

use anyhow::{Context, Ok, Result};
use clap::Parser;
use cli::{Action, CliArgs};
use thl_tools::{
    Extractor, Packer,
    csv::{all_in_one_extraction::DialogueExtractor, all_in_one_repack::DialogueRepacker},
    mbe_file::MBEFile,
};

mod cli;

fn main() -> Result<()> {
    let args = CliArgs::parse();
    args.validate()?;
    match args.action {
        Action::Extract {
            source,
            destination,
            no_rename_images,
            extract_only,
        } => {
            Extractor::new()
                .with_rename_images(!no_rename_images)
                .with_name_matcher(extract_only)
                .extract(&mut BufReader::new(File::open(&source)?), &destination)
                .context("something went wrong during the extraction")?;
        }
        Action::Pack {
            source,
            destination,
            no_rename_images,
            ..
        } => Packer::new()
            .with_rename_images(!no_rename_images)
            .pack(
                &source,
                &mut BufWriter::new(
                    File::create(&destination)
                        .with_context(|| format!("couldn't create {}", destination.display()))?,
                ),
            )
            .context("something went wrong during the repacking")?,
        Action::ReadLines { source, prefix, .. } => {
            let file = MBEFile::from_reader(&mut BufReader::new(File::open(&source)?))
                .context("something went wrong while parsing file")?;
            for (message, char_and_call) in file.into_important_messages() {
                println!(
                    "{}{}: {}",
                    prefix,
                    char_and_call.character.name(),
                    String::from_utf8_lossy(&message.text)
                )
            }
        }
        Action::ExtractDialogues {
            game_path,
            languages,
            destination,
            ..
        } => {
            DialogueExtractor::new()
                .extract(
                    &game_path,
                    &languages,
                    &mut BufWriter::new(File::open(&destination)?),
                )
                .context("error while extracting dialogues")?;
        }
        Action::RepackDialogues {
            full_text,
            reference_mvgl,
            destination,
            ..
        } => DialogueRepacker::new()
            .repack(
                &mut BufReader::new(File::open(&full_text)?),
                &mut BufReader::new(File::open(&reference_mvgl)?),
                &mut BufWriter::new(File::create(&destination)?),
            )
            .context("error while repacking dialogues")?,
    }
    Ok(())
}
