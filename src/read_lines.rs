use std::{
    io::{self, Read},
    vec::IntoIter,
};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::PlaceholderOrCharacter;

pub struct LineReader<'a> {
    source: &'a mut dyn Read,
    remaining_entries: u32,
    entry_id_difference: u32,
    last_entry_id: u32,
}

pub struct DialogueReader<'a> {
    characters: IntoIter<PlaceholderOrCharacter>,
    line_reader: LineReader<'a>,
}

trait Pushable<T> {
    fn push(&mut self, c: T);
}
impl<T> Pushable<T> for Vec<T> {
    fn push(&mut self, c: T) {
        self.push(c);
    }
}
impl<T> Pushable<T> for () {
    fn push(&mut self, _: T) {}
}

impl<'a> LineReader<'a> {
    fn new_inner(
        source: &'a mut dyn Read,
        characters: &mut dyn Pushable<PlaceholderOrCharacter>,
    ) -> io::Result<Self> {
        let mut buffer = [0; 4];
        source.read_exact(&mut buffer)?;
        assert_eq!(&buffer, b"EXPA");

        let number_of_sheets = source.read_u32::<LittleEndian>()?;
        let mut entry_id_difference = 0;

        for i in 0..number_of_sheets {
            let length_of_entry_name = source.read_u32::<LittleEndian>()?;

            let mut name = vec![0; length_of_entry_name as usize];
            source.read_exact(&mut name)?;

            let num_of_entries = source.read_u32::<LittleEndian>()?;
            for _ in 0..num_of_entries {
                source.read_u32::<LittleEndian>()?;
            }

            let length = source.read_u32::<LittleEndian>()?;
            let nb = source.read_u32::<LittleEndian>()?;
            if i == 0 {
                entry_id_difference = length;
                if (length_of_entry_name + num_of_entries * 4) % 8 != 0 {
                    let _ = source.read_u32::<LittleEndian>()?;
                }
            }

            for _ in 0..nb {
                let _id = source.read_u32::<LittleEndian>()?;
                let character = source.read_u32::<LittleEndian>()?;
                characters.push(PlaceholderOrCharacter::from(character));
                for _ in 0..length / 4 - 2 {
                    source.read_u32::<LittleEndian>()?;
                }
            }
        }

        source.read_exact(&mut buffer)?;
        assert_eq!(&buffer, b"CHNK");

        let nb_entries = source.read_u32::<LittleEndian>()?;

        Ok(Self {
            source,
            remaining_entries: nb_entries,
            entry_id_difference,
            last_entry_id: 0,
        })
    }

    pub fn new(source: &'a mut dyn Read) -> io::Result<Self> {
        Self::new_inner(source, &mut ())
    }

    pub fn next_with_entry_id(&mut self) -> Option<(Vec<u8>, u32)> {
        let (mut string_buffer, id) = loop {
            if let Some(x) = self.remaining_entries.checked_sub(1) {
                self.remaining_entries = x;
            } else {
                return None;
            }
            let id = self.source.read_u32::<LittleEndian>().ok()?;
            if self.last_entry_id == 0 {
                self.last_entry_id = id;
            }
            let string_size = self.source.read_u32::<LittleEndian>().ok()?;
            if (id - self.last_entry_id) % self.entry_id_difference == 0 {
                let mut string_buffer = vec![0; string_size as usize];
                self.last_entry_id = id;
                self.source.read_exact(&mut string_buffer).ok()?;
                break (string_buffer, id);
            } else {
                io::copy(&mut self.source.take(string_size as u64), &mut io::sink()).unwrap();
            }
        };
        while string_buffer.last() == Some(&b'\0') {
            string_buffer.pop();
        }
        Some((string_buffer, id))
    }
}

impl Iterator for LineReader<'_> {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_with_entry_id().map(|x| x.0)
    }
}

impl<'a> DialogueReader<'a> {
    pub fn new(source: &'a mut dyn Read) -> io::Result<Self> {
        let mut characters = Vec::new();
        let line_reader = LineReader::new_inner(source, &mut characters)?;
        Ok(Self {
            line_reader,
            characters: characters.into_iter(),
        })
    }
}

impl Iterator for DialogueReader<'_> {
    type Item = (PlaceholderOrCharacter, u32, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        let (next_dialogue, message_id) = self.line_reader.next_with_entry_id()?;
        let next_char = self.characters.next()?;
        Some((next_char, message_id, next_dialogue))
    }
}
