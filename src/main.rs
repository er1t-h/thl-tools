use std::{
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::Path,
};

use clap::Parser;
use cli::{Action, CliArgs};
use thl_tools::{LineReader, Translator, extract, pack};

mod cli;

fn main() {
    let args = CliArgs::parse();
    match args.action {
        Action::Extract {
            source,
            destination,
        } => {
            extract(
                &mut BufReader::new(
                    File::open(source).expect("the file to extract is should exist"),
                ),
                &destination,
            )
            .expect("something went wrong during the extraction");
        }
        Action::Pack {
            source,
            destination,
        } => pack(&source, &destination).expect("something went wrong during the repacking"),
        Action::Translate {
            source,
            mut destination,
        } => {
            let source = Path::new(&source);
            if destination.is_dir() {
                destination.push(source.file_name().expect("source should be a valid file"));
            }
            let mut source = BufReader::new(File::open(source).expect("source file should exist"));
            let mut dest = BufWriter::new(
                File::create_new(&destination).expect("destination file should not exist"),
            );
            let mut translator = Translator::new(&mut source, &mut dest);
            translator
                .translate()
                .expect("something went wrong during translation");
        }
        Action::ReadLines {
            source,
            prefix,
            ignore_duplicate,
        } => {
            let mut source =
                BufReader::new(File::open(source).expect("source should be a valid file"));
            let mut iter = LineReader::new(&mut source)
                .expect("something went wrong while fetching lines")
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
            fs::rename(&source, &new_source).expect("should be able to move the given source file");
            let mut source_file = BufReader::new(
                File::open(&new_source).expect("should be able to open source file"),
            );
            let mut destination_file = BufWriter::new(
                File::create_new(&source).expect("should be able to open new source file"),
            );
            Translator::new(&mut source_file, &mut destination_file)
                .translate()
                .expect("something went wrong during the translation");
            fs::remove_file(new_source).expect("should be able to remove given source file");
        }
    }
}
