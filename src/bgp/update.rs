use byteorder::{ByteOrder, NetworkEndian, WriteBytesExt};

use super::{BGP_TYPE_UPDATE, make_bgp_header};
use super::utils::prefix::{Prefix, compile_prefixes, extract_prefixes};
use super::utils::path_attribute::{PathAttribute, extract_path_attributes};
use crate::bgp::utils::path_attribute::compile_path_attributes;

#[derive(Debug)]
pub struct BGPUpdate {
    withdrawn_routes: Vec<Prefix>,
    path_attributes: Vec<PathAttribute>,
    network_layer_reachability_information: Vec<Prefix>,
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

        BGPUpdate {
            withdrawn_routes,
            path_attributes,
            network_layer_reachability_information: prefixes,
        }
    }
}


impl Into<Vec<u8>> for BGPUpdate {
    fn into(self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Size is a placeholder, fill later
        let header = make_bgp_header(0 as u16, BGP_TYPE_UPDATE);

        let mut withdrawn_routes = compile_prefixes(self.withdrawn_routes);
        buf.write_u16::<NetworkEndian>(withdrawn_routes.len() as u16).unwrap();
        buf.append(&mut withdrawn_routes);

        let mut path_attributes = compile_path_attributes(self.path_attributes);
        buf.write_u16::<NetworkEndian>(path_attributes.len() as u16).unwrap();
        buf.append(&mut path_attributes);

        let mut prefixes = compile_prefixes(self.network_layer_reachability_information);
        buf.append(&mut prefixes);

        // Fix size in header
        NetworkEndian::write_u16(&mut buf[16 .. 18],buf.len() as u16);

        return buf
    }
}