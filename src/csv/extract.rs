use std::{fs::File, io};

use csv::WriterBuilder;

use crate::{DialogueReader, mbe_file::MBEFile};

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
        b"Is Important",
        file_language_name.unwrap_or(b"Original"),
    ])?;
    let iter = MBEFile::parse(source)?.messages;
    for message in iter {
        let char_name = message.character.map_or("", |x| x.name());
        wtr.write_record([
            b"".as_slice(),
            char_name.as_bytes(),
            message.message_id.to_string().as_bytes(),
            message.is_important().to_string().as_bytes(),
            &message.text,
        ])?;
    }
    Ok(())
}
