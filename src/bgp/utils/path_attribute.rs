use byteorder::{ByteOrder, NetworkEndian, WriteBytesExt};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt::Formatter;

#[derive(FromPrimitive, ToPrimitive, Debug, PartialEq)]
pub enum AttributeFlag {
    Optional       = 1 << 7,
    Transitive     = 1 << 6,
    Partial        = 1 << 5,
    ExtendedLength = 1 << 4,
}

#[derive(Debug)]
pub enum AttributeType {
    Origin, ASPath, NextHop, MultiExitDisc, LocalPref, AtomicAggregate, Aggregator, Unknown(u8)
}

pub struct PathAttribute {
    flags: Vec<AttributeFlag>,
    type_code: AttributeType,
    value: Vec<u8>,
}

impl std::fmt::Debug for PathAttribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[PathAttribute] {:?}: {:?} (Flags: {:?})", self.type_code, self.value, self.flags))
    }
}

impl From<u8> for AttributeType {
    fn from(code: u8) -> Self {
        match code {
            1 => AttributeType::Origin,
            2 => AttributeType::ASPath,
            3 => AttributeType::NextHop,
            4 => AttributeType::MultiExitDisc,
            5 => AttributeType::LocalPref,
            6 => AttributeType::AtomicAggregate,
            7 => AttributeType::Aggregator,
            n => AttributeType::Unknown(n)
        }
    }
}

fn extract_attribute_flags(flags_bitfield: u8) -> Vec<AttributeFlag> {
    let mut flags = Vec::new();
    for offset in 4 .. 8 {
        let flag_bit = 1 << offset;
        if flags_bitfield & flag_bit != 0 {
            let flag: Option<AttributeFlag> = FromPrimitive::from_u8(flag_bit);
            if let Some(flag) = flag {
                flags.push(flag);
            } else {
                panic!(format!("Bad attribute flags: {}", flags_bitfield));
            }
        }
    }
    return flags
}

fn compile_attribute_flags(flags: Vec<AttributeFlag>) -> u8 {
    let mut bitfield = 0u8;
    for flag in flags {
        bitfield |= flag as u8;
    }
    return bitfield
}

pub(crate) fn extract_path_attributes(data: &[u8]) -> Vec<PathAttribute> {
    let mut path_attributes = Vec::new();

    let mut bytes_left = data.len();
    let mut i = 0;
    while bytes_left > 0 {
        let flags = extract_attribute_flags(data[i]);
        let type_code = data[i+1];

        let attribute_length;
        let attribute_header_length;
        if flags.contains(&AttributeFlag::ExtendedLength) { // Extended length
            attribute_length = NetworkEndian::read_u16(&data[i+2..i+4]) as usize;
            attribute_header_length = 4;
        } else { // Normal length
            attribute_length = data[i+2] as usize;
            attribute_header_length = 3;
        }
        let attribute_value = data[i + attribute_header_length .. i + attribute_header_length + attribute_length].to_vec();
        path_attributes.push(
            PathAttribute {
                flags,
                type_code: type_code.into(),
                value: attribute_value,
            }
        );
        i += attribute_header_length + attribute_length;
        bytes_left -= attribute_header_length + attribute_length;
    }
    return path_attributes;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_attribute_flags() {
        assert_eq!(extract_attribute_flags(0), vec![]);
        assert_eq!(extract_attribute_flags(0b1000 << 4), vec![AttributeFlag::Optional]);
        assert_eq!(extract_attribute_flags(0b0100 << 4), vec![AttributeFlag::Transitive]);
        assert_eq!(extract_attribute_flags(0b0010 << 4), vec![AttributeFlag::Partial]);
        assert_eq!(extract_attribute_flags(0b0001 << 4), vec![AttributeFlag::ExtendedLength]);
        assert_eq!(extract_attribute_flags(0b1111 << 4), vec![
            AttributeFlag::ExtendedLength,
            AttributeFlag::Partial,
            AttributeFlag::Transitive,
            AttributeFlag::Optional,
        ]);
    }

    #[test]
    fn test_compile_attribute_flags() {
        assert_eq!(0, compile_attribute_flags(vec![]));
        assert_eq!(0b1000 << 4, compile_attribute_flags(vec![AttributeFlag::Optional]));
        assert_eq!(0b0100 << 4, compile_attribute_flags(vec![AttributeFlag::Transitive]));
        assert_eq!(0b0010 << 4, compile_attribute_flags(vec![AttributeFlag::Partial]));
        assert_eq!(0b0001 << 4, compile_attribute_flags(vec![AttributeFlag::ExtendedLength]));
        assert_eq!(0b1111 << 4, compile_attribute_flags(vec![
            AttributeFlag::ExtendedLength,
            AttributeFlag::Partial,
            AttributeFlag::Transitive,
            AttributeFlag::Optional,
        ]));
    }
}