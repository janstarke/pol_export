use std::io::{Read, Seek};

use binread::{derive_binread, BinRead, BinReaderExt, BinResult, ReadOptions};
use derive_getters::Getters;
use encoding_rs::{ISO_8859_15, UTF_16LE};
use serde::Serialize;

use crate::{key_value_datatype::KeyValueDataType, registry_value::RegistryValue};

/// https://learn.microsoft.com/en-us/previous-versions/windows/desktop/policy/registry-policy-file-format
#[derive(Getters, Serialize)]
#[derive_binread]
#[br(little)]
pub struct PolicyFileEntry {
    #[br(assert(_begin == '['), parse_with=read_char)]
    #[getter(skip)]
    #[serde(skip_serializing)]
    _begin: char,

    #[br(parse_with=read_sz_string)]
    pol_key: String,

    #[br(assert(_sep1 == ';'), parse_with=read_char)]
    #[getter(skip)]
    #[serde(skip_serializing)]
    _sep1: char,

    #[br(parse_with=read_sz_string)]
    pol_value: String,

    #[br(assert(_sep2 == ';'), parse_with=read_char)]
    #[getter(skip)]
    #[serde(skip_serializing)]
    _sep2: char,

    pol_type: KeyValueDataType,

    #[br(assert(_sep3 == ';'), parse_with=read_char)]
    #[getter(skip)]
    #[serde(skip_serializing)]
    _sep3: char,

    pol_size: u32,

    #[br(assert(_sep4 == ';'), parse_with=read_char)]
    #[getter(skip)]
    #[serde(skip_serializing)]
    _sep4: char,

    #[br(parse_with=read_data, args(&pol_type, pol_size))]
    pol_data: RegistryValue,

    #[br(assert(_end == ']'), parse_with=read_char)]
    #[getter(skip)]
    #[serde(skip_serializing)]
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

fn read_char<R: Read + Seek>(reader: &mut R, _ro: &ReadOptions, _args: ()) -> BinResult<char> {
    let b: Vec<u16> = vec![reader.read_le()?];
    Ok(char::decode_utf16(b.into_iter())
        .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
        .next()
        .unwrap())
}

fn read_vec<S: Read + Seek, I: TryInto<usize>>(reader: &mut S, bytes: I) -> BinResult<Vec<u8>>
where
    <I as std::convert::TryInto<usize>>::Error: std::fmt::Debug,
{
    let mut bytes = vec![0u8; TryInto::try_into(bytes).unwrap()];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn read_data<R: Read + Seek>(
    reader: &mut R,
    _ro: &ReadOptions,
    args: (&KeyValueDataType, u32),
) -> BinResult<RegistryValue> {
    let datasize: usize = TryInto::try_into(args.1).unwrap();
    let data = match args.0 {
        KeyValueDataType::RegNone => RegistryValue::RegNone,
        KeyValueDataType::RegSZ => {
            RegistryValue::RegSZ(parse_reg_sz_raw(&read_vec(reader, datasize)?[..])?)
        }
        KeyValueDataType::RegExpandSZ => {
            RegistryValue::RegExpandSZ(parse_reg_sz_raw(&read_vec(reader, datasize)?[..])?)
        }
        KeyValueDataType::RegBinary => {
            let mut bytes = vec![0u8; TryInto::try_into(datasize).unwrap()];
            reader.read_exact(&mut bytes)?;
            RegistryValue::RegBinary(bytes)
        }
        KeyValueDataType::RegDWord => RegistryValue::RegDWord(reader.read_le()?),
        KeyValueDataType::RegDWordBigEndian => RegistryValue::RegDWordBigEndian(reader.read_be()?),
        KeyValueDataType::RegLink => {
            RegistryValue::RegLink(parse_reg_sz_raw(&read_vec(reader, datasize)?[..])?)
        }
        KeyValueDataType::RegMultiSZ => {
            let bytes = read_vec(reader, datasize)?;
            let strings = parse_reg_multi_sz(&bytes[..])?;
            RegistryValue::RegMultiSZ(strings)
        }
        KeyValueDataType::RegResourceList => todo!(),
        KeyValueDataType::RegFullResourceDescriptor => todo!(),
        KeyValueDataType::RegResourceRequirementsList => todo!(),
        KeyValueDataType::RegQWord => RegistryValue::RegQWord(reader.read_le()?),
        KeyValueDataType::RegFileTime => todo!(),
    };
    Ok(data)
}

//TODO: use function from nt_hive2
pub(crate) fn parse_reg_multi_sz(raw_string: &[u8]) -> BinResult<Vec<String>> {
    let mut multi_string: Vec<String> = parse_reg_sz_raw(raw_string)?
        .split('\0')
        .map(|x| x.to_owned())
        .collect();

    // due to the way split() works we have an empty string after the last \0 character
    // and due to the way RegMultiSZ works we have an additional empty string between the
    // last two \0 characters.
    // those additional empty strings will be deleted afterwards:
    assert!(!multi_string.len() >= 2);
    //assert_eq!(multi_string.last().unwrap().len(), 0);
    multi_string.pop();

    if multi_string.last().is_some() {
        // assert_eq!(multi_string.last().unwrap().len(), 0);
        multi_string.pop();
    }

    Ok(multi_string)
}

//TODO: use function from nt_hive2
pub fn parse_reg_sz_raw(raw_string: &[u8]) -> BinResult<String> {
    let (cow, _, had_errors) = UTF_16LE.decode(raw_string);

    if !had_errors {
        Ok(cow.strip_suffix('\0').unwrap_or(&cow).to_owned())
    } else {
        let (cow, _, had_errors) = ISO_8859_15.decode(raw_string);
        if had_errors {
            Err(binread::error::Error::Custom {
                pos: 0,
                err: Box::new("unable to decode RegSZ string"),
            })
        } else {
            //assert_eq!(raw_string.len(), cow.len());
            Ok(cow.strip_suffix('\0').unwrap_or(&cow).to_owned())
        }
    }
}

fn read_sz_string<R: Read + Seek> (
    reader: &mut R,
    _ro: &ReadOptions,
    _args: (),
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
