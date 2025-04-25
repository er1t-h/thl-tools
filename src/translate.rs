use std::io::{self, Read, Seek, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rustyline::error::ReadlineError;

pub trait Helper: Read + Seek {}
impl<T: Read + Seek> Helper for T {}

pub struct Translator<'a> {
    source: &'a mut dyn Helper,
    destination: &'a mut dyn Write,
}

impl<'a> Translator<'a> {
    pub fn new(source: &'a mut dyn Helper, destination: &'a mut dyn Write) -> Self {
        Self {
            source,
            destination,
        }
    }

    pub fn translate(&mut self) -> io::Result<()> {
        let mut buffer = [0; 4];
        self.source.read_exact(&mut buffer)?;
        assert_eq!(&buffer, b"EXPA");
        self.destination.write_all(b"EXPA")?;

        let number_of_sheets = self.source.read_u32::<LittleEndian>()?;
        self.destination
            .write_u32::<LittleEndian>(number_of_sheets)?;

        for i in 0..number_of_sheets {
            if i != 0 {
                self.source.read_exact(&mut buffer)?;
                self.destination.write_all(&buffer)?;
            }

            let length_of_entry_name = self.source.read_u32::<LittleEndian>()?;
            self.destination
                .write_u32::<LittleEndian>(length_of_entry_name)?;

            let mut name = vec![0; length_of_entry_name as usize];
            self.source.read_exact(&mut name)?;
            self.destination.write_all(&name)?;

            let num_of_entries = self.source.read_u32::<LittleEndian>()?;
            self.destination.write_u32::<LittleEndian>(num_of_entries)?;
            for _ in 0..num_of_entries {
                let val = self.source.read_u32::<LittleEndian>()?;
                self.destination.write_u32::<LittleEndian>(val)?;
            }

            let length = self.source.read_u32::<LittleEndian>()?;
            let nb = self.source.read_u32::<LittleEndian>()?;
            self.destination.write_u32::<LittleEndian>(length)?;
            self.destination.write_u32::<LittleEndian>(nb)?;

            for _ in 0..nb * (length / 4) {
                let val = self.source.read_u32::<LittleEndian>()?;
                self.destination.write_u32::<LittleEndian>(val)?;
            }
        }

        self.source.read_exact(&mut buffer)?;
        assert_eq!(&buffer, b"CHNK");
        self.destination.write_all(b"CHNK")?;

        let nb_entries = self.source.read_u32::<LittleEndian>()?;
        self.destination.write_u32::<LittleEndian>(nb_entries)?;

        let mut rl = rustyline::DefaultEditor::new().unwrap();
        let mut finish = false;
        let mut last_line = vec![];
        for _ in 0..nb_entries {
            let unk = self.source.read_u32::<LittleEndian>()?;
            self.destination.write_u32::<LittleEndian>(unk)?;

            let string_size = self.source.read_u32::<LittleEndian>()?;
            let mut string_buffer = vec![0; string_size as usize];
            self.source.read_exact(&mut string_buffer)?;
            if !finish && string_buffer != last_line && !string_buffer.starts_with(b"pic_voice") {
                let slice = if let Some(pos) = string_buffer.iter().position(|&x| x == 0) {
                    &string_buffer[..pos]
                } else {
                    &string_buffer
                };

                println!("Line to translate: {}", String::from_utf8_lossy(slice));
                match rl.readline("> ") {
                    Ok(x) if x.is_empty() => {
                        self.destination.write_u32::<LittleEndian>(string_size)?;
                        self.destination.write_all(&string_buffer)?;
                    }
                    Ok(x) => {
                        let pad = 4 - x.len() % 4;
                        let len = x.len() + pad;
                        self.destination.write_u32::<LittleEndian>(len as u32)?;
                        self.destination.write_all(x.as_bytes())?;
                        self.destination.write_all(&[0; 4][..pad])?;
                        rl.add_history_entry(x).unwrap();
                    }
                    Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                        finish = true;
                        self.destination.write_u32::<LittleEndian>(string_size)?;
                        self.destination.write_all(&string_buffer)?;
                    }
                    Err(e) => {
                        eprintln!("error while using readline: {e}");
                        finish = true;
                        self.destination.write_u32::<LittleEndian>(string_size)?;
                        self.destination.write_all(&string_buffer)?;
                    }
                }
                last_line = string_buffer;
            } else {
                self.destination.write_u32::<LittleEndian>(string_size)?;
                self.destination.write_all(&string_buffer)?;
            }
        }

        Ok(())
    }
}
