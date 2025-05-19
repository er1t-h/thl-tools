use std::io::{self, Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num::FromPrimitive;

use crate::{
    Character,
    offset_wrapper::{OffsetReadWrapper, OffsetWriteWrapper},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MBEFile {
    pub sheets: Vec<Sheet>,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sheet {
    pub name: Vec<u8>,
    pub unknown_entries: Vec<u32>,
    pub message_id_difference: u32,
    pub number_of_important: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub character: Option<Character>,
    pub call_id: Option<u32>,
    pub message_id: u32,
    pub text: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportantMessage {
    pub character: Character,
    pub call_id: u32,
    pub message_id: u32,
    pub text: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageImportanceDeterminer {
    last_offset: u32,
    message_id_difference: u32,
    should_skip_first: bool,
}

impl MessageImportanceDeterminer {
    fn new(message_id_difference: u32, should_skip_first: bool) -> Self {
        Self {
            last_offset: 0,
            message_id_difference,
            should_skip_first,
        }
    }

    fn is_important(&mut self, id: u32) -> bool {
        if self.should_skip_first {
            self.should_skip_first = false;
            false
        } else if self.last_offset == 0 {
            self.last_offset = id;
            true
        } else {
            id.checked_sub(self.last_offset)
                .is_some_and(|res| res % self.message_id_difference == 0)
        }
    }
}

impl Sheet {
    fn get_important_message_determiner(&self) -> MessageImportanceDeterminer {
        //let (l, r) = match self.unknown_entries.as_slice() {
        //    [2, 2, 7, 8, 7, 7, 7, 7, 7, 7, 7, 7] => (0x0, self.message_id_difference),
        //    [8, 7, 7, 7, 7, 7, 7] => (0x20, self.message_id_difference),
        //    [2, 2, 7, 8, 7, 7, 7, 7, 7] => (0x10, 0x8),
        //    [2, 7, 7, 7, 7, 7, 7] => (0x8, 0x8),
        //    [2, 2, 7, 8] => (0x20, self.message_id_difference),
        //    x => todo!("not implemented for {x:?}"),
        //};
        MessageImportanceDeterminer::new(
            self.message_id_difference,
            self.unknown_entries == [8, 7, 7, 7, 7, 7, 7],
        )
    }

    fn parse(source: &mut OffsetReadWrapper) -> io::Result<(Sheet, Vec<(u32, Character)>)> {
        let length_of_entry_name = source.read_u32::<LittleEndian>()?;

        let mut name = vec![0; length_of_entry_name as usize];
        source.read_exact(&mut name)?;

        let num_of_entries = source.read_u32::<LittleEndian>()?;
        let ident = (0..num_of_entries)
            .map(|_| source.read_u32::<LittleEndian>())
            .collect::<Result<Vec<_>, _>>()?;

        let length = source.read_u32::<LittleEndian>()?;
        let nb = source.read_u32::<LittleEndian>()?;

        //if i == 0 && (length_of_entry_name + num_of_entries * 4) % 8 != 0 {
        if source.offset() % 8 != 0 {
            let _ = source.read_u32::<LittleEndian>()?;
        }

        let mut call_id_and_characters = Vec::with_capacity(nb as usize);
        for _ in 0..nb {
            let id = source.read_u32::<LittleEndian>()?;
            let character = source.read_u32::<LittleEndian>()?;
            call_id_and_characters.push((id, Character::from_u32(character).unwrap()));
            for _ in 0..length / 4 - 2 {
                source.read_u32::<LittleEndian>()?;
            }
        }

        Ok((
            Sheet {
                name,
                unknown_entries: ident,
                number_of_important: nb,
                message_id_difference: length,
            },
            call_id_and_characters,
        ))
    }

    pub fn write(
        &self,
        destination: &mut OffsetWriteWrapper,
        messages: impl Iterator<Item = (u32, Character)>,
    ) -> io::Result<()> {
        let pad_bytes = 4 - self.name.len() % 4;
        let entry_name_length = self.name.len() + pad_bytes;
        destination.write_u32::<LittleEndian>(entry_name_length as u32)?;
        destination.write_all(&self.name)?;
        for _ in 0..pad_bytes {
            destination.write_u8(0)?;
        }

        destination.write_u32::<LittleEndian>(self.unknown_entries.len() as u32)?;
        for &entry in &self.unknown_entries {
            destination.write_u32::<LittleEndian>(entry)?;
        }

        destination.write_u32::<LittleEndian>(self.message_id_difference)?;
        destination.write_u32::<LittleEndian>(self.number_of_important)?;

        if destination.offset() % 8 != 0 {
            destination.write_u32::<LittleEndian>(0)?;
        }

        for (call_id, character) in messages {
            destination.write_u32::<LittleEndian>(call_id)?;
            destination.write_u32::<LittleEndian>(character as u32)?;
            for _ in 0..self.message_id_difference / 4 - 2 {
                destination.write_u32::<LittleEndian>(0)?;
            }
        }

        Ok(())
    }
}

impl Message {
    fn parse(
        source: &mut OffsetReadWrapper,
        mut characters: impl Iterator<Item = (u32, Character)>,
        importance_determiner: &mut MessageImportanceDeterminer,
    ) -> io::Result<Message> {
        let (mut text, message_id) = {
            let id = source.read_u32::<LittleEndian>()?;
            let string_size = source.read_u32::<LittleEndian>()?;
            let mut string_buffer = vec![0; string_size as usize];
            source.read_exact(&mut string_buffer)?;
            (string_buffer, id)
        };
        while text.last() == Some(&b'\0') {
            text.pop();
        }
        let (call_id, character) = if importance_determiner.is_important(message_id) {
            eprintln!("important_id: {message_id}");
            let (id, c) = characters.next().unwrap();
            (Some(id), Some(c))
        } else {
            (None, None)
        };
        Ok(Self {
            character,
            call_id,
            message_id,
            text,
        })
    }

    pub fn is_important(&self) -> bool {
        self.character.is_some()
    }

    pub fn try_into_important(self) -> Option<ImportantMessage> {
        match self {
            Self {
                character: Some(character),
                call_id: Some(call_id),
                message_id,
                text,
            } => Some(ImportantMessage {
                character,
                call_id,
                message_id,
                text,
            }),
            _ => None,
        }
    }

    pub fn write(&self, destination: &mut OffsetWriteWrapper) -> io::Result<()> {
        destination.write_u32::<LittleEndian>(self.message_id)?;
        let string_pad = 4 - self.text.len() % 4;
        let string_len = self.text.len() + string_pad;
        destination.write_u32::<LittleEndian>(string_len as u32)?;
        destination.write_all(&self.text)?;
        for _ in 0..string_pad {
            destination.write_u8(0)?;
        }
        Ok(())
    }
}

impl MBEFile {
    pub fn parse(source: &mut dyn Read) -> io::Result<MBEFile> {
        let mut source = OffsetReadWrapper::new(source);
        let mut buffer = [0; 4];
        source.read_exact(&mut buffer)?;
        assert_eq!(&buffer, b"EXPA");

        let number_of_sheets = source.read_u32::<LittleEndian>()?;

        let sheets = (0..number_of_sheets)
            .map(|_| Sheet::parse(&mut source))
            .collect::<Result<Vec<_>, _>>()?;

        source.read_exact(&mut buffer)?;
        assert_eq!(&buffer, b"CHNK");

        let (sheets, characters): (Vec<_>, Vec<_>) = sheets.into_iter().unzip();
        let nb_characters = characters.iter().map(|x| x.len()).sum::<usize>();
        let mut characters = characters.into_iter().flatten();

        let nb_messages = source.read_u32::<LittleEndian>()?;
        let mut importance_determiner = sheets[0].get_important_message_determiner();
        let messages = (0..nb_messages)
            .map(|_| Message::parse(&mut source, &mut characters, &mut importance_determiner))
            .collect::<Result<Vec<_>, _>>()?;

        if nb_characters <= nb_messages as usize {
            debug_assert!(characters.next().is_none());
        }

        Ok(Self { sheets, messages })
    }

    pub fn write(&self, destination: &mut dyn Write) -> io::Result<()> {
        let mut destination = OffsetWriteWrapper::new(destination);

        write!(destination, "EXPA")?;
        destination.write_u32::<LittleEndian>(self.sheets.len() as u32)?;
        for sheet in &self.sheets {
            sheet.write(
                &mut destination,
                self.messages
                    .iter()
                    .flat_map(|x| x.call_id.zip(x.character)),
            )?;
        }

        write!(destination, "CHNK")?;
        destination.write_u32::<LittleEndian>(self.messages.len() as u32)?;

        for message in &self.messages {
            message.write(&mut destination)?;
        }

        Ok(())
    }
}
