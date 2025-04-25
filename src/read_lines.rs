use std::io::{self, Read};

use byteorder::{LittleEndian, ReadBytesExt};

pub struct LineReader<'a> {
    source: &'a mut dyn Read,
    remaining_entries: u32,
}

impl<'a> LineReader<'a> {
    pub fn new(source: &'a mut dyn Read) -> io::Result<Self> {
        let mut buffer = [0; 4];
        source.read_exact(&mut buffer)?;
        assert_eq!(&buffer, b"EXPA");

        let number_of_sheets = source.read_u32::<LittleEndian>()?;

        for i in 0..number_of_sheets {
            if i != 0 {
                source.read_exact(&mut buffer)?;
            }

            let length_of_entry_name = source.read_u32::<LittleEndian>()?;

            let mut name = vec![0; length_of_entry_name as usize];
            source.read_exact(&mut name)?;

            let num_of_entries = source.read_u32::<LittleEndian>()?;
            for _ in 0..num_of_entries {
                source.read_u32::<LittleEndian>()?;
            }

            let length = source.read_u32::<LittleEndian>()?;
            let nb = source.read_u32::<LittleEndian>()?;

            for _ in 0..nb * (length / 4) {
                source.read_u32::<LittleEndian>()?;
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
}

impl Iterator for LineReader<'_> {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        let string_buffer = loop {
            if let Some(x) = self.remaining_entries.checked_sub(1) {
                self.remaining_entries = x;
            } else {
                return None;
            }
            let _unk = self.source.read_u32::<LittleEndian>().ok()?;
            let string_size = self.source.read_u32::<LittleEndian>().ok()?;
            let mut string_buffer = vec![0; string_size as usize];
            self.source.read_exact(&mut string_buffer).ok()?;
            if !string_buffer.starts_with(b"pic_voice") {
                break string_buffer;
            }
        };
        Some(string_buffer)
    }
}
