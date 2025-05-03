use std::{borrow::Cow, fs::File, io, path::Path};

use csv::WriterBuilder;

use crate::LineReader;

pub fn extract_as_csv(
    source: &mut File,
    destination: &File,
    translated_name: Option<&[u8]>,
    file_language_name: Option<&[u8]>,
) -> io::Result<()> {
    let mut wtr = WriterBuilder::new().from_writer(destination);
    wtr.write_record([
        translated_name.unwrap_or(b"Translated"),
        file_language_name.unwrap_or(b"Original"),
    ])?;
    let mut iter = LineReader::new(source)?.peekable();
    while let Some(line) = iter.next() {
        while iter.next_if_eq(&line).is_some() {}
        wtr.write_record([b"".as_slice(), &line])?;
    }
    Ok(())
}
