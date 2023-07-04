use std::fmt::Display;

use serde::Serialize;

pub enum RegistryValue {
    RegNone,
    #[allow(dead_code)]
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

impl Serialize for RegistryValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        match self {
            RegistryValue::RegNone => serializer.serialize_none(),
            RegistryValue::RegUnknown => serializer.serialize_none(),
            RegistryValue::RegSZ(s) => serializer.serialize_str(s),
            RegistryValue::RegExpandSZ(s) => serializer.serialize_str(s),
            RegistryValue::RegBinary(b) => serializer.serialize_str(&hex::encode(b)),
            RegistryValue::RegDWord(n) => serializer.serialize_u32(*n),
            RegistryValue::RegDWordBigEndian(n) => serializer.serialize_u32(*n),
            RegistryValue::RegLink(s) => serializer.serialize_str(s),
            RegistryValue::RegMultiSZ(ms) => serializer.serialize_str(&ms.join(",")),
            RegistryValue::RegResourceList(s) => serializer.serialize_str(s),
            RegistryValue::RegFullResourceDescriptor(s) => serializer.serialize_str(s),
            RegistryValue::RegResourceRequirementsList(_) => todo!(),
            RegistryValue::RegQWord(n) => serializer.serialize_u64(*n),
            RegistryValue::RegFileTime => todo!(),
        }
    }
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
