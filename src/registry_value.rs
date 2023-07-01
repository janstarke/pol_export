use std::fmt::Display;

use serde::Serialize;

#[derive(Serialize)]
pub enum RegistryValue {
    RegNone,
    RegUnknown,
    RegSZ(String),
    RegExpandSZ(String),
    RegBinary(Vec<u8>),
    RegDWord(u32),
    RegDWordBigEndian(u32),
    RegLink(String),
    RegMultiSZ(Vec<String>),
    RegResourceList(String),
    RegFullResourceDescriptor(String),
    RegResourceRequirementsList(String),
    RegQWord(u64),
    RegFileTime,
}

impl Display for RegistryValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryValue::RegUnknown => write!(f, "Unknown"),
            RegistryValue::RegNone => write!(f, "None"),
            RegistryValue::RegSZ(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegExpandSZ(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegBinary(val) => {
                write!(f, "{:?}", if val.len() > 16 { &val[..16] } else { val })
            }
            RegistryValue::RegDWord(val) => write!(f, "0x{:08x}", val),
            RegistryValue::RegDWordBigEndian(val) => write!(f, "0x{:08x}", val),
            RegistryValue::RegLink(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegMultiSZ(val) => write!(f, "{:?}", val),
            RegistryValue::RegResourceList(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegFullResourceDescriptor(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegResourceRequirementsList(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegQWord(val) => write!(f, "0x{:016x}", val),
            RegistryValue::RegFileTime => todo!(),
        }
    }
}
