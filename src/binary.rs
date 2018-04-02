// WebAssembly Binary Encoding Reference: https://github.com/WebAssembly/design/blob/master/BinaryEncoding.md

use byteorder::{LittleEndian, ReadBytesExt};
use leb128;
use std::fs::File;
use std::io::{Error, Read};

#[derive(Debug)]
pub enum ParseError {
    BadMagic(u32),
    UnsupportedVersion(u32),
    InvalidValueType(i8),
    IoError(Error),
    DecodeError(leb128::read::Error),
}

#[derive(Debug)]
pub struct Module {
    magic_number: u32,
    version: u32,
    sections: Vec<Section>,
}

#[derive(Debug)]
enum Section {
    Custom,
    Function { types: Vec<u32> },
    Memory { entries: Vec<MemoryType> },
    Start { index: u32 },
    Code { bodies: Vec<FunctionBody> },
    Unknown { id: u32 },
}

#[derive(Debug)]
struct MemoryType {
    limits: ResizableLimits,
}

#[derive(Debug)]
struct ResizableLimits {
    initial: u32,
    maximum: Option<u32>,
}

#[derive(Debug)]
struct FunctionBody {
    locals: Vec<LocalEntry>,
    code: Vec<u8>,
}

#[derive(Debug)]
struct LocalEntry {
    count: u32,
    ty: ValueType,
}

#[derive(Debug)]
enum ValueType {
    I32,
    I64,
    F32,
    F64,
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
            let section = try!(Section::parse(f));
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
        let id = match Section::parse_varuint32(f) {
            Err(_) => return Ok(None),
            Ok(val) => val,
        };
        let payload_len = try!(Section::parse_varuint32(f)) as usize;
        match id {
            0 => Section::parse_custom_section(f, payload_len),
            3 => Section::parse_function_section(f),
            8 => Section::parse_start_section(f),
            5 => Section::parse_memory_section(f),
            10 => Section::parse_code_section(f),
            _ => Section::parse_unknown_section(f, id, payload_len),
        }
    }

    fn parse_custom_section(
        f: &mut File,
        payload_len: usize,
    ) -> Result<Option<Section>, ParseError> {
        let name_len = try!(Section::parse_varuint32(f));
        let mut name = vec![0u8; name_len as usize];
        if let Err(e) = f.read_exact(&mut name) {
            return Err(ParseError::IoError(e));
        }
        let mut payload = vec![0u8; payload_len as usize];
        if let Err(e) = f.read_exact(&mut payload) {
            return Err(ParseError::IoError(e));
        }
        Ok(Some(Section::Custom))
    }

    fn parse_function_section(f: &mut File) -> Result<Option<Section>, ParseError> {
        let mut types = vec![];
        let count = try!(Section::parse_varuint32(f));
        for _ in 0..count {
            let ty = try!(Section::parse_varuint32(f));
            types.push(ty);
        }
        Ok(Some(Section::Function { types: types }))
    }

    fn parse_memory_section(f: &mut File) -> Result<Option<Section>, ParseError> {
        let mut entries = vec![];
        let count = try!(Section::parse_varuint32(f));
        for _ in 0..count {
            let entry = try!(Section::parse_memory_type(f));
            entries.push(entry);
        }
        Ok(Some(Section::Memory { entries: entries }))
    }

    fn parse_start_section(f: &mut File) -> Result<Option<Section>, ParseError> {
        let index = try!(Section::parse_varuint32(f));
        Ok(Some(Section::Start {
            index: index as u32,
        }))
    }

    fn parse_code_section(f: &mut File) -> Result<Option<Section>, ParseError> {
        let mut bodies = vec![];
        let count = try!(Section::parse_varuint32(f));
        for _ in 0..count {
            let body = try!(Section::parse_function_body(f));
            bodies.push(body);
        }
        Ok(Some(Section::Code { bodies: bodies }))
    }

    fn parse_function_body(f: &mut File) -> Result<FunctionBody, ParseError> {
        let _body_size = try!(Section::parse_varuint32(f));
        let mut locals = vec![];
        let local_count = try!(Section::parse_varuint32(f));
        for _ in 0..local_count {
            let local = try!(Section::parse_local_entry(f));
            locals.push(local);
        }
        let mut code = vec![];
        loop {
            let mut buf = [0; 1];
            if let Err(e) = f.read_exact(&mut buf) {
                return Err(ParseError::IoError(e));
            }
            if buf[0] == 0x0b {
                break;
            }
            code.push(buf[0]);
        }
        Ok(FunctionBody {
            locals: locals,
            code: code,
        })
    }

    fn parse_local_entry(f: &mut File) -> Result<LocalEntry, ParseError> {
        let count = try!(Section::parse_varuint32(f));
        let ty = try!(Section::parse_value_type(f));
        Ok(LocalEntry {
            count: count,
            ty: ty,
        })
    }

    fn parse_unknown_section(
        f: &mut File,
        id: u32,
        payload_len: usize,
    ) -> Result<Option<Section>, ParseError> {
        let mut payload = vec![0u8; payload_len as usize];
        if let Err(e) = f.read_exact(&mut payload) {
            return Err(ParseError::IoError(e));
        }
        Ok(Some(Section::Unknown { id: id }))
    }

    fn parse_memory_type(f: &mut File) -> Result<MemoryType, ParseError> {
        let limits = try!(Section::parse_resizable_limits(f));
        Ok(MemoryType { limits: limits })
    }

    fn parse_resizable_limits(f: &mut File) -> Result<ResizableLimits, ParseError> {
        let flags = try!(Section::parse_varuint32(f));
        let initial = try!(Section::parse_varuint32(f));
        let maximum = if flags == 1 {
            let maximum_raw = try!(Section::parse_varuint32(f));
            Some(maximum_raw)
        } else {
            None
        };
        Ok(ResizableLimits {
            initial: initial,
            maximum: maximum,
        })
    }

    fn parse_value_type(f: &mut File) -> Result<ValueType, ParseError> {
        let ty = try!(Section::parse_varint7(f));
        match ty {
            -0x01 => Ok(ValueType::I32),
            -0x02 => Ok(ValueType::I64),
            -0x03 => Ok(ValueType::F32),
            -0x04 => Ok(ValueType::F64),
            _ => Err(ParseError::InvalidValueType(ty)),
        }
    }

    fn parse_varuint32(f: &mut File) -> Result<u32, ParseError> {
        match leb128::read::signed(f) {
            Err(e) => return Err(ParseError::DecodeError(e)),
            Ok(val) => return Ok(val as u32),
        }
    }

    fn parse_varint7(f: &mut File) -> Result<i8, ParseError> {
        match leb128::read::signed(f) {
            Err(e) => return Err(ParseError::DecodeError(e)),
            Ok(val) => return Ok(val as i8),
        }
    }
}
