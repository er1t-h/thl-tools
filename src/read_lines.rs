use std::io::{self, Read};

use byteorder::{LittleEndian, ReadBytesExt};

pub struct LineReader<'a> {
    source: &'a mut dyn Read,
    remaining_entries: u32,
    markers_allowed: bool,
}

impl<'a> LineReader<'a> {
    /// There are a lot of line containing only "pic_voice", "r00_*" that this iterator ignores by default.
    pub fn allow_markers(self, is_allowed: bool) -> Self {
        Self {
            markers_allowed: is_allowed,
            ..self
        }
    }

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
        // For some reason, some files seem to have an extra u32 0
        if &buffer == b"\0\0\0\0" {
            source.read_exact(&mut buffer)?;
        }
        assert_eq!(&buffer, b"CHNK");

        let nb_entries = source.read_u32::<LittleEndian>()?;

        Ok(Self {
            source,
            remaining_entries: nb_entries,
            markers_allowed: false,
        })
    }
}

impl Iterator for LineReader<'_> {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut string_buffer = loop {
            if let Some(x) = self.remaining_entries.checked_sub(1) {
                self.remaining_entries = x;
            } else {
                return None;
            }
            let _unk = self.source.read_u32::<LittleEndian>().ok()?;
            let string_size = self.source.read_u32::<LittleEndian>().ok()?;
            let mut string_buffer = vec![0; string_size as usize];
            self.source.read_exact(&mut string_buffer).ok()?;
            if self.markers_allowed
                || !string_buffer
                    .iter()
                    .take_while(|&&x| x != 0)
                    .all(|&x| x.is_ascii_alphanumeric() || x == b'_')
            {
                break string_buffer;
            }
        };
        while string_buffer.pop_if(|&mut x| x == 0).is_some() {}
        Some(string_buffer)
    }
}
