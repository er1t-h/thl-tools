#![allow(dead_code)]

use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    fs::File,
    io::{self, BufReader, Cursor, ErrorKind, Read, Write},
    path::Path,
};

use byte_string::{ByteStr, ByteString};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num::FromPrimitive;
use num_derive::FromPrimitive;

use crate::helpers::offset_wrapper::{OffsetReadWrapper, OffsetWriteWrapper};

type Row = Vec<TableCell>;
type CreateRow<'a> = Vec<TableCreateCell<'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum ColumnType {
    Int = 2,
    IntID = 9,
    Byte = 4,
    Float = 5,
    String = 7,
    StringID = 8,
}

impl ColumnType {
    pub fn size(self) -> usize {
        match self {
            Self::StringID | Self::String => 8,
            Self::Int | Self::IntID | Self::Float => 4,
            Self::Byte => 1,
        }
    }

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
    pub fn type_(self) -> ColumnType {
        match self {
            Self::Int(_) => ColumnType::Int,
            Self::IntID(_) => ColumnType::IntID,
            Self::StringID(_) => ColumnType::StringID,
            Self::String(_) => ColumnType::String,
            Self::Float(_) => ColumnType::Float,
            Self::Byte(_) => ColumnType::Byte,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableCreateCell<'a> {
    Int(u32),
    IntID(u32),
    Byte(u8),
    Float(f32),
    String(Cow<'a, [u8]>),
    StringID(Cow<'a, [u8]>),
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

    fn unwrap_u32(self) -> u32 {
        match self {
            Self::Int(x) | Self::IntID(x) => x,
            _ => panic!("tried to unwrap value {self:?}"),
        }
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

impl Display for PublicTableCell<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(x) | Self::IntID(x) => write!(f, "{x}"),
            Self::Float(x) => write!(f, "{x}"),
            Self::Byte(x) => write!(f, "{x}"),
            Self::String(Some(x)) | Self::StringID(Some(x)) => {
                write!(f, "{}", String::from_utf8_lossy(x))
            }
            Self::String(None) | Self::StringID(None) => Ok(()),
        }
    }
}

impl<'a> PublicTableCell<'a> {
    pub fn unwrap_string(self) -> Option<&'a ByteStr> {
        match self {
            Self::String(x) => x,
            _ => panic!("tried to unwrap value {self:?}"),
        }
    }
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

impl Display for ParseMBEFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadEXPAMagicNumber => write!(f, "expected EXPA as a magic number"),
            Self::InvalidColumnType => write!(f, "specified column type is not valid"),
            Self::BadCHNKMagicNumber => write!(f, "expected CHNK as a magic number"),
            Self::Io(x) => write!(f, "io error: {x}"),
        }
    }
}
impl std::error::Error for ParseMBEFileError {}

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

pub struct RowIterator<'a> {
    file: &'a MBEFile,
    sheet: usize,
    row: usize,
}

impl RowIterator<'_> {
    pub fn sheet(&self) -> usize {
        self.sheet
    }

    pub fn row(&self) -> usize {
        self.row
    }
}

impl<'a> Iterator for RowIterator<'a> {
    type Item = Vec<PublicTableCell<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let current_sheet = self.file.get_sheet_by_index(self.sheet)?;
        match current_sheet.get_row(self.row) {
            Some(x) => {
                self.row += 1;
                Some(x.content())
            }
            None => {
                self.sheet += 1;
                self.row = 0;
                self.next()
            }
        }
    }
}

impl MBEFile {
    pub fn from_path(source: impl AsRef<Path>) -> Result<Self, ParseMBEFileError> {
        let mut file = BufReader::new(File::open(source)?);
        Self::parse(&mut OffsetReadWrapper::new(&mut file))
    }

    pub fn rows(&self) -> RowIterator {
        RowIterator {
            file: self,
            sheet: 0,
            row: 0,
        }
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
                source.align(8)?;
                let rows = (0..row_number)
                    .map(|_| {
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

        source.align(8)?;
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

    pub fn patch(mut self, mut patch: MBEFile) -> Option<MBEFile> {
        for (i, original_sheet) in self.sheets.iter_mut().enumerate() {
            let Some(new_sheet) = patch
                .sheets
                .iter_mut()
                .find(|x| x.name == original_sheet.name)
            else {
                log::info!(
                    "patch: skipping sheet {} because the patch doesn't contain a similarly named sheet",
                    i + 1
                );
                continue;
            };

            if original_sheet.column_types != new_sheet.column_types {
                return None;
            }

            let mut new_rows = Vec::new();
            let original_rows = original_sheet.rows.drain(..);
            let mut patch_rows = new_sheet.rows.drain(..).peekable();
            for original_row in original_rows {
                let id = original_row[0].unwrap_u32();
                while let Some(patch_row) = patch_rows.next_if(|row| row[0].unwrap_u32() < id) {
                    new_rows.push(patch_row);
                }
                if let Some(patch_row) = patch_rows.next_if(|row| row[0].unwrap_u32() == id) {
                    new_rows.push(patch_row);
                } else {
                    new_rows.push(original_row);
                }
            }
        }
        todo!()
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

    pub fn modify_string(
        &mut self,
        sheet: usize,
        row: usize,
        column: usize,
        new_content: Vec<u8>,
    ) -> Option<()> {
        let mut offset = 8;
        for skipped_sheet in self.sheets.iter().take(sheet) {
            offset += skipped_sheet.name.len().next_multiple_of(4)
                + 4 * (skipped_sheet.column_types.len() + 4);
            offset = offset.next_multiple_of(8);
            let mut one_line = 0_usize;
            for col_type in &skipped_sheet.column_types {
                one_line = one_line.next_multiple_of(col_type.alignment() as usize);
                one_line += col_type.size();
            }
            offset += skipped_sheet.rows.len() * one_line;
        }

        let sheet = self.sheets.get(sheet)?;
        offset += (sheet.name.len() + 1).next_multiple_of(4) + 4 * (sheet.column_types.len() + 4);
        offset = offset.next_multiple_of(8);

        let mut one_line = 0_usize;
        let mut up_to_column = 0_usize;
        for (i, col_type) in sheet.column_types.iter().enumerate() {
            one_line = one_line.next_multiple_of(col_type.alignment() as usize);
            if i == column {
                up_to_column = one_line;
            }
            one_line += col_type.size();
        }
        offset += row * one_line + up_to_column;
        if let Ok(x) = self.data.binary_search_by_key(&(offset as u32), |x| x.0) {
            self.data[x].1 = ByteString(new_content);
            Some(())
        } else {
            None
        }
    }

    pub fn write(&self, writer: &mut OffsetWriteWrapper) -> io::Result<()> {
        writer.write_all(b"EXPA")?;
        writer.write_u32::<LittleEndian>(self.sheets.len() as u32)?;

        for sheet in &self.sheets {
            write_size_prefixed_string(ByteStr::new(&sheet.name), writer, 0)?;
            writer.write_u32::<LittleEndian>(sheet.column_types.len() as u32)?;
            let mut sink = io::sink();
            let mut length = OffsetWriteWrapper::new(&mut sink);
            for type_ in &sheet.column_types {
                writer.write_u32::<LittleEndian>(*type_ as u32)?;
                length.align(type_.alignment(), 0)?;
                match type_ {
                    ColumnType::StringID | ColumnType::String => {
                        length.write_u64::<LittleEndian>(0)?
                    }
                    ColumnType::Float | ColumnType::Int | ColumnType::IntID => {
                        length.write_u32::<LittleEndian>(0)?
                    }
                    ColumnType::Byte => length.write_u8(0)?,
                }
            }
            writer.write_u32::<LittleEndian>(length.offset() as u32)?;
            writer.write_u32::<LittleEndian>(sheet.rows.len() as u32)?;
            writer.align(8, 0)?;
            for column in sheet.rows.iter().flatten() {
                writer.align(column.type_().alignment(), 0xcc)?;
                match column {
                    &TableCell::Int(x) | &TableCell::IntID(x) => {
                        writer.write_u32::<LittleEndian>(x)?
                    }
                    &TableCell::Byte(x) => writer.write_u8(x)?,
                    &TableCell::Float(x) => writer.write_f32::<LittleEndian>(x)?,
                    TableCell::String(_) | TableCell::StringID(_) => {
                        writer.write_u64::<LittleEndian>(0)?;
                    }
                }
            }
        }

        writer.align(8, 0)?;
        writer.write_all(b"CHNK")?;
        writer.write_u32::<LittleEndian>(self.data.len() as u32)?;

        for (id, string) in &self.data {
            writer.write_u32::<LittleEndian>(*id)?;
            write_size_prefixed_string(ByteStr::new(string), writer, 1)?;
        }

        Ok(())
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

    pub fn get_file(&self) -> &'a MBEFile {
        self.file
    }

    pub fn column_types(self) -> &'a [ColumnType] {
        &self.file.sheets[self.sheet_index].column_types
    }

    pub fn number_of_row(&self) -> usize {
        self.file
            .sheets
            .get(self.sheet_index)
            .map_or(0, |x| x.rows.len())
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

pub struct SheetCreator<'a> {
    pub name: ByteString,
    pub column_types: Vec<ColumnType>,
    pub rows: Vec<CreateRow<'a>>,
}

pub struct MBEFileCreator<'a> {
    pub sheets: Vec<SheetCreator<'a>>,
}

fn write_size_prefixed_string(
    str: &ByteStr,
    writer: &mut OffsetWriteWrapper,
    nul_byte: usize,
) -> std::io::Result<()> {
    let len = str.len() + nul_byte; // accounting for nul byte
    let pad = 4 - len % 4;
    let padded_len = len + pad;
    writer.write_u32::<LittleEndian>(padded_len as u32)?;
    writer.write_all(str)?;
    io::copy(&mut io::repeat(0).take((pad + nul_byte) as u64), writer)?;
    Ok(())
}

impl MBEFileCreator<'_> {
    pub fn write(&self, writer: &mut OffsetWriteWrapper) -> io::Result<()> {
        writer.write_all(b"EXPA")?;
        writer.write_u32::<LittleEndian>(self.sheets.len() as u32)?;
        let mut data = Vec::new();

        for sheet in &self.sheets {
            write_size_prefixed_string(ByteStr::new(&sheet.name), writer, 0)?;
            writer.write_u32::<LittleEndian>(sheet.column_types.len() as u32)?;
            let mut sink = io::sink();
            let mut length = OffsetWriteWrapper::new(&mut sink);
            for type_ in &sheet.column_types {
                writer.write_u32::<LittleEndian>(*type_ as u32)?;
                length.align(type_.alignment(), 0)?;
                match type_ {
                    ColumnType::StringID | ColumnType::String => {
                        length.write_u64::<LittleEndian>(0)?
                    }
                    ColumnType::Float | ColumnType::Int | ColumnType::IntID => {
                        length.write_u32::<LittleEndian>(0)?
                    }
                    ColumnType::Byte => length.write_u8(0)?,
                }
            }
            length.align(4, 0)?;
            writer.write_u32::<LittleEndian>(length.offset() as u32)?;
            writer.write_u32::<LittleEndian>(sheet.rows.len() as u32)?;
            writer.align(8, 0xcc)?;
            for column in sheet.rows.iter().flatten() {
                match column {
                    &TableCreateCell::Int(x) | &TableCreateCell::IntID(x) => {
                        writer.write_u32::<LittleEndian>(x)?
                    }
                    &TableCreateCell::Byte(x) => writer.write_u8(x)?,
                    &TableCreateCell::Float(x) => writer.write_f32::<LittleEndian>(x)?,
                    TableCreateCell::String(x) | TableCreateCell::StringID(x) => {
                        data.push((writer.offset() as u32, ByteStr::new(x)));
                        writer.write_u64::<LittleEndian>(0)?;
                    }
                }
            }
        }

        writer.write_all(b"CHNK")?;
        writer.write_u32::<LittleEndian>(data.len() as u32)?;

        for (id, string) in data {
            writer.write_u32::<LittleEndian>(id)?;
            write_size_prefixed_string(string, writer, 1)?;
        }

        Ok(())
    }
}
