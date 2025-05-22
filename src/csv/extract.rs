use std::io::{self, Read, Write};

use csv::Writer;

use crate::mbe::MBEFile;

pub fn extract_as_csv(
    source: &mut dyn Read,
    destination: &mut Writer<&mut dyn Write>,
    translated_name: Option<&[u8]>,
    file_language_name: Option<&[u8]>,
) -> io::Result<()> {
    destination.write_record([
        translated_name.unwrap_or(b"Translated"),
        b"Character Name",
        b"Entry ID",
        b"Is Important",
        file_language_name.unwrap_or(b"Original"),
    ])?;
    let file = MBEFile::from_reader(source)?;
    for message in file.into_messages() {
        match message {
            (message, None) => destination.write_record([
                b"".as_slice(),
                b"",
                message.message_id.to_string().as_bytes(),
                b"false",
                &message.text,
            ])?,
            (message, Some(char_and_call)) => {
                destination.write_record([
                    b"".as_slice(),
                    char_and_call.character.name().as_bytes(),
                    message.message_id.to_string().as_bytes(),
                    b"true",
                    &message.text,
                ])?;
            }
        };
    }
    Ok(())
}
