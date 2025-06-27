use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    PlaceholderOrCharacter,
    helpers::offset_wrapper::{OffsetReadWrapper, OffsetWriteWrapper},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharAndCallId {
    pub character: PlaceholderOrCharacter,
    pub call_id: u32,
}

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
    pub char_and_calls: Vec<CharAndCallId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
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

    fn parse(source: &mut OffsetReadWrapper) -> io::Result<Sheet> {
        let length_of_entry_name = source.read_u32::<LittleEndian>()?;

        let mut name = vec![0; length_of_entry_name as usize];
        source.read_exact(&mut name)?;
        while name.pop_if(|&mut x| x == 0).is_some() {}

        let num_of_entries = source.read_u32::<LittleEndian>()?;
        let ident = (0..num_of_entries)
            .map(|_| source.read_u32::<LittleEndian>())
            .collect::<Result<Vec<_>, _>>()?;

        let length = source.read_u32::<LittleEndian>()?;
        let nb = source.read_u32::<LittleEndian>()?;

        if source.offset() % 8 != 0 {
            let _ = source.read_u32::<LittleEndian>()?;
        }

        let mut call_id_and_characters = Vec::with_capacity(nb as usize);
        for _ in 0..nb {
            let id = source.read_u32::<LittleEndian>()?;
            let character = source.read_u32::<LittleEndian>()?;
            call_id_and_characters.push(CharAndCallId {
                call_id: id,
                character: PlaceholderOrCharacter::from(character),
            });
            for _ in 0..length / 4 - 2 {
                source.read_u32::<LittleEndian>()?;
            }
        }

        Ok(Sheet {
            name,
            unknown_entries: ident,
            char_and_calls: call_id_and_characters,
            message_id_difference: length,
        })
    }

    pub fn write(&self, destination: &mut OffsetWriteWrapper) -> io::Result<()> {
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
        destination.write_u32::<LittleEndian>(self.char_and_calls.len() as u32)?;

        if destination.offset() % 8 != 0 {
            destination.write_u32::<LittleEndian>(0)?;
        }

        for &CharAndCallId { character, call_id } in &self.char_and_calls {
            destination.write_u32::<LittleEndian>(call_id)?;
            destination.write_u32::<LittleEndian>(character.into())?;
            for _ in 0..self.message_id_difference / 4 - 2 {
                destination.write_u32::<LittleEndian>(0)?;
            }
        }

        Ok(())
    }
}

impl Message {
    fn parse(source: &mut OffsetReadWrapper) -> io::Result<Message> {
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
        Ok(Self { message_id, text })
    }

    pub fn write(&self, destination: &mut OffsetWriteWrapper) -> io::Result<()> {
        destination.write_u32::<LittleEndian>(self.message_id)?;
        let len_with_nul = self.text.len() + 1;
        let string_pad = 4 - len_with_nul % 4;
        let string_len = len_with_nul + string_pad;
        destination.write_u32::<LittleEndian>(string_len as u32)?;
        destination.write_all(&self.text)?;
        for _ in 0..string_pad + 1 {
            destination.write_u8(0)?;
        }
        Ok(())
    }
}

impl MBEFile {
    pub fn from_reader(source: &mut dyn Read) -> io::Result<Self> {
        Self::parse_inner(&mut OffsetReadWrapper::new(source))
    }
    pub fn from_path(path: &dyn AsRef<Path>) -> io::Result<Self> {
        let mut file = BufReader::new(File::open(path)?);
        Self::parse_inner(&mut OffsetReadWrapper::new(&mut file))
    }

    pub fn new(sheet_names: Vec<Vec<u8>>) -> Self {
        Self {
            sheets: sheet_names
                .into_iter()
                .map(|x| Sheet {
                    char_and_calls: vec![],
                    name: x,
                    unknown_entries: vec![2, 2, 7, 8, 7, 7, 7, 7, 7, 7, 7, 7],
                    message_id_difference: 0x38,
                })
                .collect(),
            messages: vec![],
        }
    }

    pub fn add_message(&mut self, message: Vec<u8>, character_and_id: CharAndCallId) {
        let message_id = if let Some(message) = self.messages.last() {
            message.message_id + self.sheets[0].message_id_difference
        } else {
            self.sheets[0].message_id_difference
        };
        self.messages.push(Message {
            message_id,
            text: message,
        });
        self.sheets[0].char_and_calls.push(character_and_id);
    }

    pub fn into_important_messages(self) -> impl Iterator<Item = (Message, CharAndCallId)> {
        self.into_messages().flat_map(|(message, char_and_call)| {
            char_and_call.map(|char_and_call| (message, char_and_call))
        })
    }

    pub fn into_messages(self) -> impl Iterator<Item = (Message, Option<CharAndCallId>)> {
        IntoMessage {
            importance_determiner: self.sheets[0].get_important_message_determiner(),
            char_and_calls: self.sheets.into_iter().flat_map(|x| x.char_and_calls),
            messages: self.messages.into_iter(),
        }
    }

    fn parse_inner(source: &mut OffsetReadWrapper) -> io::Result<MBEFile> {
        let mut buffer = [0; 4];
        source.read_exact(&mut buffer)?;
        assert_eq!(&buffer, b"EXPA");

        let number_of_sheets = source.read_u32::<LittleEndian>()?;

        let sheets = (0..number_of_sheets)
            .map(|_| Sheet::parse(source))
            .collect::<Result<Vec<_>, _>>()?;

        source.read_exact(&mut buffer)?;
        assert_eq!(&buffer, b"CHNK");

        let nb_messages = source.read_u32::<LittleEndian>()?;
        let messages = (0..nb_messages)
            .map(|_| Message::parse(source))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { sheets, messages })
    }

    pub fn write(&self, destination: &mut dyn Write) -> io::Result<()> {
        let mut destination = OffsetWriteWrapper::new(destination);

        write!(destination, "EXPA")?;
        destination.write_u32::<LittleEndian>(self.sheets.len() as u32)?;
        for sheet in &self.sheets {
            sheet.write(&mut destination)?;
        }

        write!(destination, "CHNK")?;
        destination.write_u32::<LittleEndian>(self.messages.len() as u32)?;

        for message in &self.messages {
            message.write(&mut destination)?;
        }

        Ok(())
    }
}

pub struct IntoMessage<MI, CACI> {
    messages: MI,
    char_and_calls: CACI,
    importance_determiner: MessageImportanceDeterminer,
}

impl<MI, CACI> Iterator for IntoMessage<MI, CACI>
where
    MI: Iterator<Item = Message>,
    CACI: Iterator<Item = CharAndCallId>,
{
    type Item = (Message, Option<CharAndCallId>);
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.messages.next()?;
        if self.importance_determiner.is_important(next.message_id) {
            let char_and_call = self.char_and_calls.next()?;
            Some((next, Some(char_and_call)))
        } else {
            Some((next, None))
        }
    }
}

pub struct IntoMessageMut<MI, CACI> {
    messages: MI,
    char_and_calls: CACI,
    importance_determiner: MessageImportanceDeterminer,
}

impl<'a, MI, CACI> Iterator for IntoMessageMut<MI, CACI>
where
    MI: Iterator<Item = &'a mut Message>,
    CACI: Iterator<Item = &'a mut CharAndCallId>,
{
    type Item = (&'a mut Message, Option<&'a mut CharAndCallId>);
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.messages.next()?;
        if self.importance_determiner.is_important(next.message_id) {
            let char_and_call = self.char_and_calls.next()?;
            Some((next, Some(char_and_call)))
        } else {
            Some((next, None))
        }
    }
}
