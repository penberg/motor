// WebAssembly Binary Encoding Reference: https://github.com/WebAssembly/design/blob/master/BinaryEncoding.md

use std::fs::File;
use std::io::{Error, Read};
use byteorder::{LittleEndian, ReadBytesExt};
use leb128;

#[derive(Debug)]
pub enum ParseError {
    BadMagic(u32),
    UnsupportedVersion(u32),
    IoError(Error),
    DecodeError(leb128::read::Error),
}

#[derive(Debug)]
enum SectionType {
    Custom = 0,
    Type = 1,
    Import = 2,
    Function = 3,
    Table = 4,
    Memory = 5,
    Global = 6,
    Export = 7,
    Start = 8,
    Element = 9,
    Code = 10,
    Data = 11,
    Unknown,
}

#[derive(Debug)]
pub struct Module {
    magic_number: u32,
    version: u32,
    sections: Vec<Section>,
}

#[derive(Debug)]
pub struct Section {
    section_type: SectionType,
}

impl Module {
    pub fn parse(f: &mut File) -> Result<Module, ParseError> {
        let magic_number = f.read_u32::<LittleEndian>().unwrap();
        if magic_number != 0x6d736100 {
            return Err(ParseError::BadMagic(magic_number));
        }
        let version = f.read_u32::<LittleEndian>().unwrap();
        if version != 0x01 {
            return Err(ParseError::UnsupportedVersion(version));
        }
        let mut sections = vec![];
        loop {
            let section = match Section::parse(f) {
                Err(e) => return Err(e),
                Ok(section) => section,
            };
            if section.is_none() {
                break;
            }
            sections.push(section.unwrap());
        }
        return Ok(Module {
            magic_number: magic_number,
            version: version,
            sections: sections,
        });
    }
}

impl Section {
    fn parse(f: &mut File) -> Result<Option<Section>, ParseError> {
        let id_result = leb128::read::signed(f);
        if id_result.is_err() {
            return Ok(None);
        }
        let id = id_result.unwrap();
        let section_type = match id {
            0 => SectionType::Custom,
            1 => SectionType::Type,
            2 => SectionType::Import,
            3 => SectionType::Function,
            4 => SectionType::Table,
            5 => SectionType::Memory,
            6 => SectionType::Global,
            7 => SectionType::Export,
            8 => SectionType::Start,
            9 => SectionType::Element,
            10 => SectionType::Code,
            11 => SectionType::Data,
            _ => SectionType::Unknown,
        };
        let payload_len = match leb128::read::signed(f) {
            Err(e) => return Err(ParseError::DecodeError(e)),
            Ok(val) => val,
        };
        if id == 0 {
            let name_len = match leb128::read::signed(f) {
                Err(e) => return Err(ParseError::DecodeError(e)),
                Ok(val) => val,
            };
            let mut name = vec![0u8; name_len as usize];
            if let Err(e) = f.read_exact(&mut name) {
                return Err(ParseError::IoError(e));
            }
        }
        let mut payload = vec![0u8; payload_len as usize];
        if let Err(e) = f.read_exact(&mut payload) {
            return Err(ParseError::IoError(e));
        }
        Ok(Some(Section { section_type: section_type }))
    }
}
