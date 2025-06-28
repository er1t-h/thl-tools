use std::{
    borrow::Cow,
    io::{self, Read, Write},
};

use csv::Writer;

use crate::{
    PlaceholderOrCharacter,
    helpers::offset_wrapper::OffsetReadWrapper,
    mbe::{MBEFile, TableCell},
};

pub fn extract_as_csv(
    source: &mut dyn Read,
    destination: &mut Writer<&mut dyn Write>,
    translated_name: Option<&[u8]>,
    file_language_name: Option<&[u8]>,
) -> io::Result<()> {
    destination.write_record([
        b"Call ID".as_slice(),
        b"Character Name",
        translated_name.unwrap_or(b"Translated"),
        file_language_name.unwrap_or(b"Original"),
    ])?;
    let file = MBEFile::parse(&mut OffsetReadWrapper::new(source)).unwrap();
    for row in file.rows() {
        let (character, message) = match row.get(1) {
            Some(&TableCell::Int(x)) => (
                PlaceholderOrCharacter::from(x).name(),
                row[2].unwrap_string(),
            ),
            _ => (Cow::Borrowed(""), row[1].unwrap_string()),
        };
        destination.write_record([
            row[0].to_string().as_bytes(),
            character.as_bytes(),
            b"",
            message.map_or(b"", |x| &x.0),
        ])?;
    }
    Ok(())
}
