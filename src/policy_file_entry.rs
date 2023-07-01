use std::io::{Read, Seek};

use binread::{derive_binread, BinRead, BinReaderExt, BinResult, ReadOptions};
use derive_getters::Getters;
use serde::Serialize;

use crate::{key_value_datatype::KeyValueDataType, registry_value::RegistryValue};

/// https://learn.microsoft.com/en-us/previous-versions/windows/desktop/policy/registry-policy-file-format
#[derive(Getters, Serialize)]
#[derive_binread]
#[br(little)]
pub struct PolicyFileEntry {
    #[br(assert(_begin == '['), parse_with=read_char)]
    #[getter(skip)]
    #[serde(skip)]
    _begin: char,

    #[br(parse_with=read_sz_string)]
    pol_key: String,

    #[br(assert(_sep1 == ';'), parse_with=read_char)]
    #[getter(skip)]
    _sep1: char,

    #[br(parse_with=read_sz_string)]
    pol_value: String,

    #[br(assert(_sep2 == ';'), parse_with=read_char)]
    #[getter(skip)]
    _sep2: char,

    pol_type: KeyValueDataType,

    #[br(assert(_sep3 == ';'), parse_with=read_char)]
    #[getter(skip)]
    _sep3: char,

    pol_size: u32,

    #[br(assert(_sep4 == ';'), parse_with=read_char)]
    #[getter(skip)]
    _sep4: char,

    #[br(parse_with=read_data, args(&pol_type, pol_size))]
    pol_data: RegistryValue,

    #[br(assert(_end == ']'), parse_with=read_char)]
    #[getter(skip)]
    _end: char,
}

fn words_to_string_lossy(words: Vec<u16>) -> String {
    char::decode_utf16(words.into_iter())
        .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}

#[derive(BinRead)]
#[br(import(length: u32))]
struct WordArray(#[br(count=length)] Vec<u16>);

fn read_char<R: Read + Seek>(reader: &mut R, _ro: &ReadOptions, args: ()) -> BinResult<char> {
    let b: Vec<u16> = vec![reader.read_le()?];
    Ok(char::decode_utf16(b.into_iter())
        .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
        .next()
        .unwrap())
}

fn read_data<R: Read + Seek>(
    reader: &mut R,
    _ro: &ReadOptions,
    args: (&KeyValueDataType, u32),
) -> BinResult<RegistryValue> {
    let datasize = &args.1;
    let data = match args.0 {
        KeyValueDataType::RegNone => RegistryValue::RegNone,
        KeyValueDataType::RegSZ => {
            let words: WordArray = reader.read_le_args((*datasize,))?;
            let s = words_to_string_lossy(words.0);
            RegistryValue::RegSZ(s)
        }
        KeyValueDataType::RegExpandSZ => todo!(),
        KeyValueDataType::RegBinary => todo!(),
        KeyValueDataType::RegDWord => RegistryValue::RegDWord(reader.read_le()?),
        KeyValueDataType::RegDWordBigEndian => RegistryValue::RegDWord(reader.read_be()?),
        KeyValueDataType::RegLink => todo!(),
        KeyValueDataType::RegMultiSZ => todo!(),
        KeyValueDataType::RegResourceList => todo!(),
        KeyValueDataType::RegFullResourceDescriptor => todo!(),
        KeyValueDataType::RegResourceRequirementsList => todo!(),
        KeyValueDataType::RegQWord => RegistryValue::RegQWord(reader.read_le()?),
        KeyValueDataType::RegFileTime => todo!(),
    };
    Ok(data)
}

fn read_sz_string<R: Read + Seek>(
    reader: &mut R,
    _ro: &ReadOptions,
    args: (),
) -> BinResult<String> {
    let mut words: Vec<u16> = Vec::new();
    loop {
        let b: u16 = reader.read_le()?;
        if b == 0 {
            break;
        }
        words.push(b)
    }
    Ok(words_to_string_lossy(words))
}
