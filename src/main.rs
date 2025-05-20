use std::{fs::File, io::BufReader};

use anyhow::{Context, Ok, Result};
use clap::Parser;
use cli::{Action, CliArgs};
use csv::Reader;
use thl_tools::{extract, mbe_file::MBEFile, pack};

mod cli;

fn main() -> Result<()> {
    let args = CliArgs::parse();
    args.validate()?;
    match args.action {
        Action::Extract {
            source,
            destination,
        } => {
            extract(
                &mut BufReader::new(
                    File::open(&source)
                        .with_context(|| format!("{} should exist", source.display()))?,
                ),
                &destination,
                None,
            )
            .context("something went wrong during the extraction")?;
        }
        Action::Pack {
            source,
            destination,
        } => pack(
            &source,
            &mut File::create_new(&destination)
                .with_context(|| format!("{} shouldn't exist", destination.display()))?,
        )
        .context("something went wrong during the repacking")?,
        Action::ReadLines { source, prefix, .. } => {
            let mut source = BufReader::new(
                File::open(&source)
                    .with_context(|| format!("{} should be a valid file", source.display()))?,
            );
            let file = MBEFile::from_reader(&mut source)
                .context("something went wrong while parsing file")?;
            for (message, char_and_call) in file.into_important_messages() {
                println!(
                    "{}{} ({:5}): {}",
                    prefix,
                    char_and_call.character.name(),
                    message.message_id,
                    String::from_utf8_lossy(&message.text)
                )
            }
        }
        Action::ExtractDialogues {
            game_path,
            languages,
            destination,
            overwrite,
        } => {
            let mut file = File::options()
                .write(true)
                .truncate(true)
                .create_new(!overwrite)
                .open(&destination)
                .with_context(|| format!("couldn't open {}:", destination.display()))?;
            thl_tools::csv::all_in_one_extraction::all_in_one_extraction(
                &game_path, &languages, &mut file,
            )
            .context("error while extracting ressources to CSV")?;
        }
        Action::RepackDialogues {
            full_text,
            reference_mvgl,
            destination,
        } => thl_tools::csv::all_in_one_repack::all_in_one_repack(
            Reader::from_path(full_text)?,
            &mut File::open(reference_mvgl)?,
            &mut File::create(destination)?,
        )?,
        #[allow(unreachable_patterns)]
        _ => todo!(),
    }
    Ok(())
}
