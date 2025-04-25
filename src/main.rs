use std::{
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
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
                &mut BufReader::new(File::open(source).unwrap()),
                Path::new(&destination),
            )
            .unwrap();
        }
        Action::Pack {
            source,
            destination,
        } => pack(Path::new(&source), Path::new(&destination)).unwrap(),
        Action::Translate {
            source,
            destination,
        } => {
            let source = Path::new(&source);
            let mut pathbuf = PathBuf::from(destination);
            if pathbuf.is_dir() {
                pathbuf.push(source.file_name().unwrap());
            }
            let mut source = BufReader::new(File::open(source).unwrap());
            let mut dest = BufWriter::new(File::create_new(&pathbuf).unwrap());
            let mut translator = Translator::new(&mut source, &mut dest);
            translator.translate().unwrap();
        }
        Action::ReadLines { source, prefix } => {
            let mut source = BufReader::new(File::open(source).unwrap());
            for line in LineReader::new(&mut source).unwrap() {
                println!("{}{}", prefix, String::from_utf8_lossy(&line));
            }
        }
        Action::EditTranslate { source } => {
            let new_source = source.with_extension("tmp");
            fs::rename(&source, &new_source).expect("couldn't move old translated file");
            let mut source_file =
                BufReader::new(File::open(&new_source).expect("couldn't open old translated file"));
            let mut destination_file = BufWriter::new(
                File::create_new(&source).expect("couldn't open new translated file"),
            );
            Translator::new(&mut source_file, &mut destination_file)
                .translate()
                .expect("something went wrong during the translation");
        }
    }
}
