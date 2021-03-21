use byteorder::{ByteOrder, NetworkEndian, WriteBytesExt};
use crate::bgp::{BGP_HEADER_SIZE, BGP_OPEN_SIZE, BGP_TYPE_OPEN, make_bgp_header};

#[derive(Debug)]
pub struct BGPOpen {
    pub version: u8,
    pub sender_as: u16,
    pub hold_time: u16,
    pub bgp_id: u32,
    pub opt_params_len: u8,
    pub opt_params: ()
}

impl From<&[u8]> for BGPOpen {
    fn from(buf: &[u8]) -> BGPOpen {
        BGPOpen {
            version: buf[0],
            sender_as: NetworkEndian::read_u16(&buf[1..3]),
            hold_time: NetworkEndian::read_u16(&buf[3..5]),
            bgp_id: NetworkEndian::read_u32(&buf[5..9]),
            opt_params_len: buf[9],
            opt_params: (),
        }
    }
}

impl Into<Vec<u8>> for BGPOpen {
    fn into(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BGP_HEADER_SIZE + BGP_OPEN_SIZE);

        let header = make_bgp_header(BGP_OPEN_SIZE as u16, BGP_TYPE_OPEN);
        buf.extend_from_slice(&header[..]);
        buf.push(self.version);
        buf.write_u16::<NetworkEndian>(self.sender_as).unwrap();
        buf.write_u16::<NetworkEndian>(self.hold_time).unwrap();
        buf.write_u32::<NetworkEndian>(self.bgp_id).unwrap();
        buf.push(self.opt_params_len);
        if self.opt_params_len > 0 { unimplemented!(); }

        return buf
    }
}