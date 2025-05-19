use std::{borrow::Cow, fs::File, io};

use csv::WriterBuilder;

use crate::DialogueReader;

pub fn extract_as_csv(
    source: &mut File,
    destination: &File,
    translated_name: Option<&[u8]>,
    file_language_name: Option<&[u8]>,
) -> io::Result<()> {
    let mut wtr = WriterBuilder::new().from_writer(destination);
    wtr.write_record([
        translated_name.unwrap_or(b"Translated"),
        b"Character Name",
        b"Entry ID",
        file_language_name.unwrap_or(b"Original"),
    ])?;
    let iter = DialogueReader::new(source)?;
    for (character, entry_id, line) in iter {
        let char_name = character.map_or(Cow::Borrowed(""), |x| x.as_str());
        wtr.write_record([
            b"".as_slice(),
            char_name.as_bytes(),
            entry_id.to_string().as_bytes(),
            &line,
        ])?;
    }
    Ok(())
}
