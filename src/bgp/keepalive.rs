use crate::bgp::{BGP_HEADER_SIZE, BGP_TYPE_KEEPALIVE, make_bgp_header};

#[derive(Debug)]
pub struct BGPKeepalive {}

impl From<&[u8]> for BGPKeepalive {
    fn from(_buf: &[u8]) -> BGPKeepalive {
        BGPKeepalive {}
    }
}

impl Into<[u8; BGP_HEADER_SIZE]> for BGPKeepalive {
    fn into(self) -> [u8; BGP_HEADER_SIZE] {
        make_bgp_header(0, BGP_TYPE_KEEPALIVE)
    }
}