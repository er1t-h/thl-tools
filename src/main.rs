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
    LineReader, extract, pack,
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
            let mut wtr = WriterBuilder::new().from_writer(
                File::create_new(&destination)
                    .with_context(|| format!("{} should not exist", destination.display()))?,
            );
            wtr.write_record([b"Translated".as_slice(), b"Original".as_slice()])?;
            let mut file = File::open(source)?;
            let mut iter = LineReader::new(&mut file)
                .context("something went wrong while fetching lines")?
                .peekable();
            while let Some(line) = iter.next() {
                while iter.next_if_eq(&line).is_some() {}
                wtr.write_record([b"".as_slice(), &line])?;
            }
        }
        Action::ReintegrateCsv {
            csv_file,
            original_mbe_file,
            target,
        } => {
            let should_remove_original;
            let original = if let Some(original) = original_mbe_file {
                original
            } else {
                csv_file.with_extension("mbe")
            };
            let (original_path, destination) = if let Some(target) = target {
                should_remove_original = false;
                (original, target)
            } else {
                let tmp = original.with_extension("tmp");
                fs::rename(&original, &tmp)?;
                should_remove_original = true;
                (original, tmp)
            };

            let mut original = File::open(&original_path)
                .with_context(|| format!("{} should be openable", original_path.display()))?;
            let mut destination = File::create_new(&destination)
                .with_context(|| format!("{} should not exist", destination.display()))?;
            let mut patcher = Patcher::new(&mut original, &mut destination);
            patcher
                .patch(CSVStrategy::new(
                    ReaderBuilder::new()
                        .from_path(&csv_file)
                        .with_context(|| format!("{} should be openable", csv_file.display()))?,
                ))
                .context("error while patching the file")?;
            if should_remove_original {
                fs::remove_file(original_path)?;
            }
        }
    }
    Ok(())
}
