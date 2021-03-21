use byteorder::{ByteOrder, NetworkEndian, WriteBytesExt};
use std::fmt;

pub struct Prefix {
    length: u8,
    prefix: [u8; 4],
}

impl fmt::Debug for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{}.{}.{}.{}/{}", self.prefix[0], self.prefix[1], self.prefix[2], self.prefix[3], self.length))
    }
}

#[derive(Debug)]
pub struct BGPUpdate {
    withdrawn_routes_len: u16,
    withdrawn_routes: Vec<Prefix>,
    total_path_attribute_len: u16,
    path_attributes: Vec<PathAttribute>,
    network_layer_reachability_information: Vec<Prefix>,
}

#[derive(Debug)]
pub struct PathAttribute {
    flags: u8,
    type_code: u8,
    value: Vec<u8>,
}

fn extract_prefixes(data: &[u8]) -> Vec<Prefix> {
    let mut routes: Vec<Prefix> = Vec::new();

    let mut bytes_left = data.len();
    let mut i = 0; // Start after the "withdrawn length" field

    while bytes_left > 0 {
        let prefix_length = data[i];
        let prefix_octets = (prefix_length as f32 / 8f32).ceil() as usize;
        let mut prefix = [0 as u8; 4];
        prefix[0..prefix_octets].copy_from_slice(&data[i+1 .. i+1+prefix_octets]);
        routes.push(Prefix {
            prefix: prefix,
            length: prefix_length,
        });
        i += 1 + prefix_octets;
        bytes_left -= 1 + prefix_octets as usize;
    }

    return routes;
}

fn extract_path_attributes(data: &[u8]) -> Vec<PathAttribute> {
    let mut path_attributes = Vec::new();

    let mut bytes_left = data.len();
    let mut i = 0;
    while bytes_left > 0 {
        let flags = data[i];
        let type_code = data[i+1];

        let attribute_length;
        let attribute_header_length;
        if (flags & 0b00010000) != 0 { // Extended length
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
                type_code,
                value: attribute_value,
            }
        );
        i += attribute_header_length + attribute_length;
        bytes_left -= attribute_header_length + attribute_length;
    }
    return path_attributes;
}

const U16_LENGTH_FIELD: usize = 2;

impl From<&[u8]> for BGPUpdate {
    fn from(buf: &[u8]) -> BGPUpdate {
        let withdrawn_routes_start = 0;
        let withdrawn_length = NetworkEndian::read_u16(&buf[withdrawn_routes_start .. withdrawn_routes_start + U16_LENGTH_FIELD]);
        let withdrawn_routes = extract_prefixes(&buf[withdrawn_routes_start + U16_LENGTH_FIELD .. withdrawn_routes_start + U16_LENGTH_FIELD + withdrawn_length as usize]);

        let path_attributes_start = withdrawn_routes_start + U16_LENGTH_FIELD + withdrawn_length as usize;
        let path_attribute_length = NetworkEndian::read_u16(&buf[path_attributes_start .. path_attributes_start + U16_LENGTH_FIELD]);
        let path_attributes = extract_path_attributes(
            &buf[path_attributes_start + U16_LENGTH_FIELD .. path_attributes_start + U16_LENGTH_FIELD + path_attribute_length as usize]
        );

        let prefixes_start = path_attributes_start + U16_LENGTH_FIELD + path_attribute_length as usize;
        let prefixes_length = buf.len() - prefixes_start;
        let prefixes = extract_prefixes(&buf[prefixes_start .. prefixes_start + prefixes_length]);

        //println!("{:?}", &prefixes);

        BGPUpdate {
            withdrawn_routes_len: withdrawn_length,
            withdrawn_routes: withdrawn_routes,
            total_path_attribute_len: path_attribute_length,
            path_attributes: path_attributes,
            network_layer_reachability_information: prefixes,
        }
    }
}