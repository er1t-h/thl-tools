use std::{
    borrow::Cow,
    fs::{self, File},
    io::{BufReader, BufWriter},
};

use anyhow::{Context, Ok, Result};
use clap::Parser;
use cli::{Action, CliArgs};
use csv::{ReaderBuilder, WriterBuilder};
use rustyline::DefaultEditor;
use thl_tools::{
    LineReader,
    csv::{
        agglomerate::agglomerate_csv, extract::extract_as_csv, fuse::fuse_csv,
        reintegrate::reintegrate_csv,
    },
    extract, pack,
    translate::{CSVStrategy, Patcher, ReadlineStrategy},
};

mod cli;

fn main() -> Result<()> {
    let args = CliArgs::parse();
    args.action.validate()?;
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
            )
            .context("something went wrong during the extraction")?;
        }
        Action::Pack {
            source,
            destination,
        } => pack(&source, &destination).context("something went wrong during the repacking")?,
        Action::Translate {
            source,
            mut destination,
        } => {
            if destination.is_dir() {
                destination.push(
                    source
                        .file_name()
                        .with_context(|| format!("{} should be a valid file", source.display()))?,
                );
            }
            let mut source = BufReader::new(
                File::open(&source)
                    .with_context(|| format!("{} should exist", source.display()))?,
            );
            let mut dest = BufWriter::new(
                File::create_new(&destination)
                    .with_context(|| format!("{} should not exist", destination.display()))?,
            );
            let mut translator = Patcher::new(&mut source, &mut dest);
            translator
                .patch(ReadlineStrategy::new(
                    DefaultEditor::new().context("failed to create a rustyline editor")?,
                ))
                .context("something went wrong during translation")?;
        }
        Action::ReadLines {
            source,
            prefix,
            ignore_duplicate,
        } => {
            let mut source = BufReader::new(
                File::open(&source)
                    .with_context(|| format!("{} should be a valid file", source.display()))?,
            );
            let mut iter = LineReader::new(&mut source)
                .context("something went wrong while fetching lines")?
                .peekable();
            while let Some(line) = iter.next() {
                if ignore_duplicate {
                    while iter.next_if_eq(&line).is_some() {}
                }
                println!("{}{}", prefix, String::from_utf8_lossy(&line));
            }
        }
        Action::EditTranslate { source } => {
            let new_source = source.with_extension("tmp");
            fs::rename(&source, &new_source).with_context(|| {
                format!(
                    "should be able to move {} to {} (maybe it already exists?)",
                    source.display(),
                    new_source.display()
                )
            })?;
            let mut source_file = BufReader::new(
                File::open(&new_source)
                    .with_context(|| format!("{} should be openable", new_source.display()))?,
            );
            let mut destination_file = BufWriter::new(
                File::create_new(&source)
                    .with_context(|| format!("{} should be openable", source.display()))?,
            );
            Patcher::new(&mut source_file, &mut destination_file)
                .patch(ReadlineStrategy::new(
                    DefaultEditor::new().context("failed to create a rustyline editor")?,
                ))
                .context("something went wrong during the translation")?;
            fs::remove_file(&new_source)
                .with_context(|| format!("{} should be removable", new_source.display()))?;
        }
        Action::ExtractAsCsv {
            source,
            destination,
        } => {
            let destination = destination.unwrap_or_else(|| source.with_extension("csv"));
            extract_as_csv(
                &mut File::open(&source)
                    .with_context(|| format!("{} couldn't open", source.display()))?,
                &File::create_new(&destination)
                    .with_context(|| format!("{} should not exist", destination.display()))?,
                None,
                None,
            )
            .context("something went wrong during extraction as CSV")?;
        }
        Action::ReintegrateCsv {
            csv_file,
            original_mbe_file,
            target,
        } => {
            reintegrate_csv(&csv_file, original_mbe_file.as_deref(), target.as_deref())
                .context("something went wrong during CSV reintegration")?;
        }
        Action::AgglomerateCsv {
            directory,
            destination,
        } => agglomerate_csv(&directory, &destination).context("error while agglomerating CSVs")?,
        Action::FuseCsv {
            first_source,
            second_source,
            destination,
        } => fuse_csv(&first_source, &second_source, &destination)
            .context("error while fusing CSVs")?,
        Action::AllInOneExtract {
            game_path,
            languages,
        } => thl_tools::csv::all_in_one_extraction::all_in_one_extraction(&game_path, &languages)
            .context("error while extracting ressources to CSV")?,
        #[allow(unreachable_patterns)]
        _ => todo!(),
    }
    Ok(())
}
