use byteorder::{ByteOrder, NetworkEndian};
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

impl Into<[u8; BGP_HEADER_SIZE + BGP_OPEN_SIZE]> for BGPOpen {
    fn into(self) -> [u8; BGP_HEADER_SIZE + BGP_OPEN_SIZE] {
        let mut buf = [0 as u8; BGP_HEADER_SIZE + BGP_OPEN_SIZE];
        const BHS: usize = BGP_HEADER_SIZE;

        let header = make_bgp_header(BGP_OPEN_SIZE as u16, BGP_TYPE_OPEN);

        buf[0 .. BHS].copy_from_slice(&header[..]);
        buf[BHS + 0] = self.version;
        NetworkEndian::write_u16(&mut buf[BHS + 1 .. BHS + 3], self.sender_as);
        NetworkEndian::write_u16(&mut buf[BHS + 3 .. BHS + 5], self.hold_time);
        NetworkEndian::write_u32(&mut buf[BHS + 5 .. BHS + 9], self.bgp_id);
        buf[BHS + 9] = self.opt_params_len;

        return buf
    }
}