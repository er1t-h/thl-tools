use std::{fs::File, io};

use csv::WriterBuilder;

use crate::DialogueReader;

pub fn extract_as_csv(
    source: &mut File,
    destination: &File,
    translated_name: Option<&[u8]>,
    file_language_name: Option<&[u8]>,
    add_character_name: bool,
) -> io::Result<()> {
    let mut wtr = WriterBuilder::new().from_writer(destination);
    if add_character_name {
        wtr.write_record([
            translated_name.unwrap_or(b"Translated"),
            b"Character Name",
            file_language_name.unwrap_or(b"Original"),
        ])?;
    } else {
        wtr.write_record([
            translated_name.unwrap_or(b"Translated"),
            file_language_name.unwrap_or(b"Original"),
        ])?;
    }
    let iter = DialogueReader::new(source)?;
    for (character, line) in iter {
        if add_character_name {
            let char_name = character.as_str();
            wtr.write_record([b"".as_slice(), char_name.as_bytes(), &line])?;
        } else {
            wtr.write_record([b"".as_slice(), &line])?;
        }
    }
    Ok(())
}
