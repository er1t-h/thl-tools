use std::{
    fs::{self, File},
    io::{BufReader, BufWriter},
};

use anyhow::{Context, Ok, Result};
use clap::Parser;
use cli::{Action, CliArgs};
use thl_tools::{LineReader, Translator, extract, pack};

mod cli;

fn main() -> Result<()> {
    let args = CliArgs::parse();
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
            let mut translator = Translator::new(&mut source, &mut dest);
            translator
                .translate()
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
            Translator::new(&mut source_file, &mut destination_file)
                .translate()
                .context("something went wrong during the translation")?;
            fs::remove_file(&new_source)
                .with_context(|| format!("{} should be removable", new_source.display()))?;
        }
    }
    Ok(())
}
