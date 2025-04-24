use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use clap::Parser;
use cli::{Action, CliArgs};
use packer::{extract, pack};

mod cli;

fn main() {
    let args = CliArgs::parse();
    match args.action {
        Action::Extract => {
            extract(
                &mut BufReader::new(File::open(args.source).unwrap()),
                Path::new(&args.destination),
            )
            .unwrap();
        }
        Action::Pack => pack(&args.source, &args.destination).unwrap(),
    }
    //pack(&args.source, &args.destination).unwrap();
}
