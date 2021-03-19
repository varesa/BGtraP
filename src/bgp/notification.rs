#[derive(Debug)]
pub struct BGPNotification {
    error_code: u8,
    error_subcode: u8,
    data: Vec<u8>,
}

impl From<&[u8]> for BGPNotification {
    fn from(buf: &[u8]) -> BGPNotification {
        BGPNotification {
            error_code: buf[0],
            error_subcode: buf[1],
            data: buf[2..].to_vec(),
        }
    }
}