use std::{
    fs::File,
    io::{self, BufReader},
};

use csv::WriterBuilder;

use crate::mbe_file::MBEFile;

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
    let file = MBEFile::from_reader(&mut BufReader::new(source))?;
    for message in file.into_messages() {
        match message {
            (message, None) => wtr.write_record([
                b"".as_slice(),
                b"",
                message.message_id.to_string().as_bytes(),
                b"false",
                &message.text,
            ])?,
            (message, Some(char_and_call)) => {
                wtr.write_record([
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
