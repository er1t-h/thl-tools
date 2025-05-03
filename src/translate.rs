use std::{
    borrow::Cow,
    convert::Infallible,
    fmt::{Debug, Display},
    io::{self, Read, Seek, Write},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rustyline::{Editor, error::ReadlineError};

pub trait Helper: Read + Seek {}
impl<T: Read + Seek> Helper for T {}

pub struct Patcher<'a> {
    source: &'a mut dyn Helper,
    destination: &'a mut dyn Write,
}

pub trait GetLine {
    type Error;
    fn next_line<'a, 's>(&'s mut self, input: &'a [u8]) -> Result<Cow<'a, [u8]>, Self::Error>
    where
        's: 'a;
}

pub struct ReadlineStrategy<H: rustyline::Helper, I: rustyline::history::History> {
    rl: Editor<H, I>,
    is_finished: bool,
}

impl<H: rustyline::Helper, I: rustyline::history::History> ReadlineStrategy<H, I> {
    pub fn new(rl: Editor<H, I>) -> Self {
        Self {
            rl,
            is_finished: false,
        }
    }
}

impl<H: rustyline::Helper, I: rustyline::history::History> GetLine for ReadlineStrategy<H, I> {
    type Error = Infallible;
    fn next_line<'a, 's>(&mut self, input: &'a [u8]) -> Result<Cow<'a, [u8]>, Self::Error>
    where
        's: 'a,
    {
        if self.is_finished {
            return Ok(Cow::Borrowed(input));
        }

        println!("Line to translate: {}", String::from_utf8_lossy(input));
        match self.rl.readline("> ") {
            Ok(s) if s.is_empty() => Ok(Cow::Borrowed(input)),
            Ok(s) => {
                // Not being able to add the entry to the history isn't a big deal actually.
                let _ = self.rl.add_history_entry(s.clone());
                Ok(Cow::Owned(s.into_bytes()))
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                self.is_finished = true;
                Ok(Cow::Borrowed(input))
            }
            Err(e) => {
                eprintln!("error while using readline: {e}");
                self.is_finished = true;
                Ok(Cow::Borrowed(input))
            }
        }
    }
}
pub struct NoStrategy;
impl GetLine for NoStrategy {
    type Error = Infallible;
    fn next_line<'a, 's>(&'s mut self, input: &'a [u8]) -> Result<Cow<'a, [u8]>, Self::Error>
    where
        's: 'a,
    {
        Ok(Cow::Borrowed(input))
    }
}

pub struct CSVStrategy<R> {
    source: csv::ByteRecordsIntoIter<R>,
}
impl<R: Read> CSVStrategy<R> {
    pub fn new(reader: csv::Reader<R>) -> Self {
        Self {
            source: reader.into_byte_records(),
        }
    }
}

impl<R: Read> GetLine for CSVStrategy<R> {
    type Error = csv::Error;
    fn next_line<'a, 's>(&mut self, input: &'a [u8]) -> Result<Cow<'a, [u8]>, Self::Error>
    where
        's: 'a,
    {
        match self.source.next() {
            Some(Ok(x)) => match x.get(0) {
                Some([]) => Ok(Cow::Borrowed(input)),
                Some(x) => Ok(Cow::Owned(x.to_vec())),
                None => Ok(Cow::Borrowed(input)),
            },
            Some(Err(e)) => Err(e),
            None => Ok(Cow::Borrowed(input)),
        }
    }
}

#[derive(Debug)]
pub enum PatchError<E> {
    Io(io::Error),
    Strategy(E),
}

impl<E: Display> Display for PatchError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(io) => write!(f, "io error: {io}"),
            Self::Strategy(strategy) => write!(f, "strategy error: {strategy}"),
        }
    }
}
impl<E: Display + Debug> std::error::Error for PatchError<E> {}

impl<E> From<io::Error> for PatchError<E> {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl<'a> Patcher<'a> {
    pub fn new(source: &'a mut dyn Helper, destination: &'a mut dyn Write) -> Self {
        Self {
            source,
            destination,
        }
    }

    pub fn patch<S: GetLine>(&mut self, mut strategy: S) -> Result<(), PatchError<S::Error>> {
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
        if &buffer == b"\0\0\0\0" {
            self.destination.write_all(b"\0\0\0\0")?;
            self.source.read_exact(&mut buffer)?;
        }
        assert_eq!(&buffer, b"CHNK");
        self.destination.write_all(b"CHNK")?;

        let nb_entries = self.source.read_u32::<LittleEndian>()?;
        self.destination.write_u32::<LittleEndian>(nb_entries)?;

        let mut last_line = vec![];
        for _ in 0..nb_entries {
            let unk = self.source.read_u32::<LittleEndian>()?;
            self.destination.write_u32::<LittleEndian>(unk)?;

            let string_size = self.source.read_u32::<LittleEndian>()?;
            let mut string_buffer = vec![0; string_size as usize];
            self.source.read_exact(&mut string_buffer)?;
            if string_buffer != last_line
                && !string_buffer
                    .iter()
                    .take_while(|&&x| x != 0)
                    .all(|&x| x.is_ascii_alphanumeric() || x == b'_')
            {
                let slice = if let Some(pos) = string_buffer.iter().position(|&x| x == 0) {
                    &string_buffer[..pos]
                } else {
                    &string_buffer
                };

                match strategy.next_line(slice) {
                    Ok(x) => {
                        let pad = 4 - x.len() % 4;
                        let len = x.len() + pad;
                        self.destination.write_u32::<LittleEndian>(len as u32)?;
                        self.destination.write_all(&x)?;
                        self.destination.write_all(&[0; 4][..pad])?;
                    }
                    Err(e) => Err(PatchError::Strategy(e))?,
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
