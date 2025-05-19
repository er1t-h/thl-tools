use std::{
    io::{self, Read},
    vec::IntoIter,
};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::PlaceholderOrCharacter;

pub struct LineReader<'a> {
    source: &'a mut dyn Read,
    remaining_entries: u32,
}

pub struct DialogueReader<'a> {
    characters: IntoIter<PlaceholderOrCharacter>,
    line_reader: LineReader<'a>,
    entry_id_offset: u32,
    difference_between_entries: u32,
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
        entry_id_offset: &mut u32,
        difference_between_entries: &mut u32,
    ) -> io::Result<Self> {
        let mut buffer = [0; 4];
        source.read_exact(&mut buffer)?;
        assert_eq!(&buffer, b"EXPA");

        let number_of_sheets = source.read_u32::<LittleEndian>()?;

        for i in 0..number_of_sheets {
            let length_of_entry_name = source.read_u32::<LittleEndian>()?;

            let mut name = vec![0; length_of_entry_name as usize];
            source.read_exact(&mut name)?;

            let num_of_entries = source.read_u32::<LittleEndian>()?;
            let ident = (0..num_of_entries)
                .map(|_| source.read_u32::<LittleEndian>())
                .collect::<Result<Vec<_>, _>>()?;

            let length = source.read_u32::<LittleEndian>()?;
            let nb = source.read_u32::<LittleEndian>()?;

            if i == 0 {
                *difference_between_entries = length;
                if (length_of_entry_name + num_of_entries * 4) % 8 != 0 {
                    let _ = source.read_u32::<LittleEndian>()?;
                }

                if nb != 0 {
                    match ident.as_slice() {
                        [2, 2, 7, 8, 7, 7, 7, 7, 7, 7, 7, 7] => {
                            *entry_id_offset = 0;
                        }
                        [8, 7, 7, 7, 7, 7, 7] => {
                            *entry_id_offset = 32;
                        }
                        [2, 2, 7, 8, 7, 7, 7, 7, 7] => {
                            *entry_id_offset = 0x10;
                            *difference_between_entries = 8;
                        }
                        [2, 7, 7, 7, 7, 7, 7] => {
                            *entry_id_offset = 8;
                            *difference_between_entries = 8;
                        }
                        [2, 2, 7, 8] => {
                            *entry_id_offset = 0x20;
                        }
                        x => todo!("not implemented for {x:?}"),
                    }
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
        })
    }

    pub fn new(source: &'a mut dyn Read) -> io::Result<Self> {
        Self::new_inner(source, &mut (), &mut 0, &mut 0)
    }

    pub fn next_with_entry_id(&mut self) -> Option<(Vec<u8>, u32)> {
        let (mut string_buffer, id) = {
            if let Some(x) = self.remaining_entries.checked_sub(1) {
                self.remaining_entries = x;
            } else {
                return None;
            }
            let id = self.source.read_u32::<LittleEndian>().ok()?;
            let string_size = self.source.read_u32::<LittleEndian>().ok()?;
            let mut string_buffer = vec![0; string_size as usize];
            self.source.read_exact(&mut string_buffer).ok()?;
            (string_buffer, id)
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
        let mut entry_id_offset = 0;
        let mut difference_between_entries = 0;
        let line_reader = LineReader::new_inner(
            source,
            &mut characters,
            &mut entry_id_offset,
            &mut difference_between_entries,
        )?;
        Ok(Self {
            line_reader,
            characters: characters.into_iter(),
            entry_id_offset,
            difference_between_entries,
        })
    }
}

impl Iterator for DialogueReader<'_> {
    type Item = (Option<PlaceholderOrCharacter>, u32, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        let (next_dialogue, message_id) = self.line_reader.next_with_entry_id()?;
        let next_char =
            if (message_id - self.entry_id_offset) % self.difference_between_entries == 0 {
                self.entry_id_offset = message_id;
                Some(self.characters.next()?)
            } else {
                None
            };
        Some((next_char, message_id, next_dialogue))
    }
}
