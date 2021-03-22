use byteorder::{ByteOrder, NetworkEndian, WriteBytesExt};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt::Formatter;

macro_rules! extended_enum {
    ($name:ident, [$($value:literal: $label:ident),+ $(,)?]) => {
        #[derive(Debug, PartialEq)]
        pub enum $name {
            $(
                $label,
            )+
            Unknown(u8)
        }

        impl From<u8> for $name {
            fn from(i: u8) -> Self {
                match i {
                    $(
                        $value => $name::$label,
                    )+
                    n => $name::Unknown(n)
                }
            }
        }

        impl Into<u8> for $name {
            fn into(self) -> u8 {
                match self {
                    $(
                        $name::$label => $value,
                    )+
                    $name::Unknown(n) => n,
                }
            }
        }
    }
}

#[derive(FromPrimitive, ToPrimitive, Debug, PartialEq, Clone, Copy)]
pub enum AttributeFlag {
    Optional       = 1 << 7,
    Transitive     = 1 << 6,
    Partial        = 1 << 5,
    ExtendedLength = 1 << 4,
}

extended_enum!(AttributeType, [
    1: Origin,
    2: ASPath,
    3: NextHop,
    4: MultiExitDisc,
    5: LocalPref,
    6: AtomicAggregate,
    7: Aggregator,
]);

#[derive(PartialEq)]
pub struct PathAttribute {
    pub flags: Vec<AttributeFlag>,
    pub type_code: AttributeType,
    pub value: Vec<u8>,
}

impl std::fmt::Debug for PathAttribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[PathAttribute] {:?}: {:?} (Flags: {:?})", self.type_code, self.value, self.flags))
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

fn compile_attribute_flags(flags: &Vec<AttributeFlag>) -> u8 {
    let mut bitfield = 0u8;
    for flag in flags {
        bitfield |= *flag as u8;
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

pub(crate) fn compile_path_attributes(attributes: Vec<PathAttribute>) -> Vec<u8> {
    let mut buffer = Vec::new();
    for attribute in attributes {
        buffer.push(compile_attribute_flags(&attribute.flags));
        buffer.push(attribute.type_code.into());
        if attribute.flags.contains(&AttributeFlag::ExtendedLength) {
            buffer.write_u16::<NetworkEndian>(attribute.value.len() as u16).unwrap();
        } else {
            buffer.push(attribute.value.len() as u8);
        }
        buffer.extend(&attribute.value);
    }
    return buffer;
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
        assert_eq!(0, compile_attribute_flags(&vec![]));
        assert_eq!(0b1000 << 4, compile_attribute_flags(&vec![AttributeFlag::Optional]));
        assert_eq!(0b0100 << 4, compile_attribute_flags(&vec![AttributeFlag::Transitive]));
        assert_eq!(0b0010 << 4, compile_attribute_flags(&vec![AttributeFlag::Partial]));
        assert_eq!(0b0001 << 4, compile_attribute_flags(&vec![AttributeFlag::ExtendedLength]));
        assert_eq!(0b1111 << 4, compile_attribute_flags(&vec![
            AttributeFlag::ExtendedLength,
            AttributeFlag::Partial,
            AttributeFlag::Transitive,
            AttributeFlag::Optional,
        ]));
    }

    #[test]
    fn test_extract_path_attributes() {
        assert_eq!(extract_path_attributes(&[]), vec![]);

        assert_eq!(
            extract_path_attributes(&[/* flags */ 0, /* type code */ 0, /* length */ 0]),
            vec![PathAttribute { type_code: AttributeType::Unknown(0), value: vec![], flags: vec![] }]
        );

        assert_eq!(
            extract_path_attributes(&[/* flags */ 0b0100 << 4, /* type code */ 1, /* length */ 1, /* value */ 2]),
            vec![PathAttribute { type_code: AttributeType::Origin, value: vec![2], flags: vec![AttributeFlag::Transitive] }]
        );

        assert_eq!(
            extract_path_attributes(&[
                /* flags */ 0b0101 << 4, /* type code */ 2, /* length */ 0, 0,
                /* flags */ 0b1000 << 4, /* type code */ 4, /* length */ 4, /* value */ 0, 0, 0, 0
            ]),
            vec![
                PathAttribute { type_code: AttributeType::ASPath, value: vec![], flags: vec![AttributeFlag::ExtendedLength, AttributeFlag::Transitive] },
                PathAttribute { type_code: AttributeType::MultiExitDisc, value: vec![0, 0, 0, 0], flags: vec![AttributeFlag::Optional]},
            ]
        );
    }

    #[test]
    fn test_compile_path_attributes() {
        assert_eq!(compile_path_attributes(vec![]), vec![]);

        assert_eq!(
            compile_path_attributes(vec![PathAttribute { type_code: AttributeType::Unknown(0), value: vec![], flags: vec![] }]),
            vec![/* flags */ 0, /* type code */ 0, /* length */ 0]
        );

        assert_eq!(
            compile_path_attributes(vec![PathAttribute { type_code: AttributeType::Origin, value: vec![2], flags: vec![AttributeFlag::Transitive] }]),
            vec![/* flags */ 0b0100 << 4, /* type code */ 1, /* length */ 1, /* value */ 2]
        );

        assert_eq!(
            compile_path_attributes(vec![
                PathAttribute { type_code: AttributeType::ASPath, value: vec![], flags: vec![AttributeFlag::ExtendedLength, AttributeFlag::Transitive] },
                PathAttribute { type_code: AttributeType::MultiExitDisc, value: vec![0, 0, 0, 0], flags: vec![AttributeFlag::Optional]},
            ]),
            vec![
                /* flags */ 0b0101 << 4, /* type code */ 2, /* length */ 0, 0,
                /* flags */ 0b1000 << 4, /* type code */ 4, /* length */ 4, /* value */ 0, 0, 0, 0
            ]
        )
    }
}