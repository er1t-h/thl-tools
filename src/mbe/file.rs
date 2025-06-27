use std::{
    fmt::Debug,
    fs::File,
    io::{self, BufReader, Cursor, ErrorKind, Read},
    path::Path,
};

use byte_string::{ByteStr, ByteString};
use byteorder::{LittleEndian, ReadBytesExt};
use num::FromPrimitive;
use num_derive::FromPrimitive;

use crate::helpers::offset_wrapper::OffsetReadWrapper;

type Row = Vec<TableCell>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
enum ColumnType {
    Int = 2,
    IntID = 9,
    Byte = 4,
    Float = 5,
    String = 7,
    StringID = 8,
}

impl ColumnType {
    pub fn alignment(self) -> u64 {
        match self {
            Self::StringID | Self::String => 8,
            Self::Int | Self::IntID | Self::Float => 4,
            Self::Byte => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TableCell {
    Int(u32),
    IntID(u32),
    Byte(u8),
    Float(f32),
    String(u32),
    StringID(u32),
}

impl TableCell {
    fn parse(
        row_offset: u32,
        source: &mut OffsetReadWrapper,
        type_: ColumnType,
    ) -> io::Result<Self> {
        Ok(match type_ {
            ColumnType::Int => Self::Int(source.read_u32::<LittleEndian>()?),
            ColumnType::IntID => Self::IntID(source.read_u32::<LittleEndian>()?),
            ColumnType::Float => Self::Float(source.read_f32::<LittleEndian>()?),
            ColumnType::Byte => Self::Byte(source.read_u8()?),
            ColumnType::String => {
                let cell = Self::String(row_offset + source.offset() as u32);
                source.read_u64::<LittleEndian>()?;
                cell
            }
            ColumnType::StringID => {
                let cell = Self::StringID(row_offset + source.offset() as u32);
                source.read_u64::<LittleEndian>()?;
                cell
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PublicTableCell<'a> {
    Int(u32),
    IntID(u32),
    Byte(u8),
    Float(f32),
    String(Option<&'a ByteStr>),
    StringID(Option<&'a ByteStr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    name: ByteString,
    column_types: Vec<ColumnType>,
    rows: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MBEFile {
    sheets: Vec<Sheet>,
    data: Vec<(u32, ByteString)>,
}

#[derive(Debug)]
pub enum ParseMBEFileError {
    BadEXPAMagicNumber,
    BadCHNKMagicNumber,
    InvalidColumnType,
    Io(io::Error),
}

impl From<io::Error> for ParseMBEFileError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

fn read_size_prefixed_string_and_remove_nuls(
    source: &mut OffsetReadWrapper,
) -> io::Result<ByteString> {
    let length = source.read_u32::<LittleEndian>()? as usize;
    let mut string = vec![0; length];
    source.read_exact(&mut string)?;
    while string.pop_if(|&mut x| x == 0).is_some() {}
    Ok(ByteString(string))
}

impl MBEFile {
    pub fn from_path(source: impl AsRef<Path>) -> Result<Self, ParseMBEFileError> {
        let mut file = BufReader::new(File::open(source)?);
        Self::parse(&mut OffsetReadWrapper::new(&mut file))
    }

    pub fn parse(source: &mut OffsetReadWrapper) -> Result<Self, ParseMBEFileError> {
        let mut magic_number = [0; 4];
        source.read_exact(&mut magic_number)?;
        if &magic_number != b"EXPA" {
            return Err(ParseMBEFileError::BadEXPAMagicNumber);
        }

        let number_of_sheet = source.read_u32::<LittleEndian>()?;
        let sheets = (0..number_of_sheet)
            .map(|_| {
                let name = read_size_prefixed_string_and_remove_nuls(source)?;
                let nb_columns = source.read_u32::<LittleEndian>()?;
                let Some(column_types) = (0..nb_columns)
                    .map(|_| Ok(ColumnType::from_u32(source.read_u32::<LittleEndian>()?)))
                    .collect::<Result<Option<Vec<_>>, io::Error>>()?
                else {
                    return Err(ParseMBEFileError::InvalidColumnType);
                };
                let row_length = source.read_u32::<LittleEndian>()?;
                let row_number = source.read_u32::<LittleEndian>()?;
                let mut row_buffer = vec![0; row_length as usize];
                let rows = (0..row_number)
                    .map(|_| {
                        source.align(8)?;
                        let row_offset = source.offset() as u32;
                        source.read_exact(&mut row_buffer)?;
                        let mut source = Cursor::new(&row_buffer);
                        let mut source = OffsetReadWrapper::new(&mut source);
                        column_types
                            .iter()
                            .map(|&column_type| {
                                source.align(column_type.alignment())?;
                                TableCell::parse(row_offset, &mut source, column_type)
                            })
                            .collect::<io::Result<Vec<_>>>()
                    })
                    .collect::<io::Result<Vec<_>>>()?;
                Ok(Sheet {
                    name,
                    column_types,
                    rows,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        if matches!(source.read_exact(&mut magic_number), Err(x) if x.kind() == ErrorKind::UnexpectedEof)
        {
            return Ok(Self {
                sheets,
                data: Vec::new(),
            });
        }
        if &magic_number != b"CHNK" {
            return Err(ParseMBEFileError::BadCHNKMagicNumber);
        }
        let number_of_data = source.read_u32::<LittleEndian>()?;
        let data = (0..number_of_data)
            .map(|_| {
                let offset = source.read_u32::<LittleEndian>()?;
                let string = read_size_prefixed_string_and_remove_nuls(source)?;
                Ok((offset, string))
            })
            .collect::<Result<Vec<_>, io::Error>>()?;

        Ok(Self { sheets, data })
    }

    pub fn get_sheet_by_name(&self, name: &[u8]) -> Option<RowSelectioner<'_>> {
        let sheet_index = self.sheets.iter().position(|x| x.name.0 == name)?;
        Some(RowSelectioner {
            sheet_index,
            file: self,
        })
    }

    pub fn get_sheet_by_index(&self, index: usize) -> Option<RowSelectioner<'_>> {
        if self.sheets.len() <= index {
            None
        } else {
            Some(RowSelectioner {
                sheet_index: index,
                file: self,
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RowSelectioner<'a> {
    sheet_index: usize,
    file: &'a MBEFile,
}

impl<'a> RowSelectioner<'a> {
    pub fn get_row(self, index: usize) -> Option<ColumnSelectioner<'a>> {
        if self.file.sheets[self.sheet_index].rows.len() <= index {
            None
        } else {
            Some(ColumnSelectioner {
                sheet_index: self.sheet_index,
                row_index: index,
                file: self.file,
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColumnSelectioner<'a> {
    sheet_index: usize,
    row_index: usize,
    file: &'a MBEFile,
}

fn cell_to_public(cell: TableCell, data: &[(u32, ByteString)]) -> PublicTableCell {
    match cell {
        TableCell::Float(x) => PublicTableCell::Float(x),
        TableCell::Int(x) => PublicTableCell::Int(x),
        TableCell::IntID(x) => PublicTableCell::IntID(x),
        TableCell::Byte(x) => PublicTableCell::Byte(x),
        TableCell::String(x) => {
            if let Ok(idx) = data.binary_search_by_key(&x, |(x, _)| *x) {
                PublicTableCell::String(Some(ByteStr::new(data[idx].1.as_slice())))
            } else {
                PublicTableCell::String(None)
            }
        }
        TableCell::StringID(x) => {
            if let Ok(idx) = data.binary_search_by_key(&x, |(x, _)| *x) {
                PublicTableCell::StringID(Some(ByteStr::new(data[idx].1.as_slice())))
            } else {
                PublicTableCell::StringID(None)
            }
        }
    }
}

impl<'a> ColumnSelectioner<'a> {
    pub fn content(self) -> Vec<PublicTableCell<'a>> {
        self.file.sheets[self.sheet_index].rows[self.row_index]
            .iter()
            .map(|&cell| cell_to_public(cell, &self.file.data))
            .collect()
    }

    pub fn get_column(self, index: usize) -> Option<PublicTableCell<'a>> {
        let cell = *self.file.sheets[self.sheet_index].rows[self.row_index].get(index)?;
        Some(cell_to_public(cell, &self.file.data))
    }
}
