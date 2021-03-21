use crate::bgp::{BGP_HEADER_SIZE, BGP_TYPE_KEEPALIVE, make_bgp_header};

#[derive(Debug)]
pub struct BGPKeepalive {}

impl From<&[u8]> for BGPKeepalive {
    fn from(_buf: &[u8]) -> BGPKeepalive {
        BGPKeepalive {}
    }
}

impl Into<Vec<u8>> for BGPKeepalive {
    fn into(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BGP_HEADER_SIZE);
        let header = make_bgp_header(0 as u16, BGP_TYPE_KEEPALIVE);
        buf.extend_from_slice(&header[..]);
        return buf
    }
}