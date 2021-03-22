pub mod update;
pub mod open;
pub mod keepalive;
pub mod notification;
pub mod errors;
pub mod utils;

use byteorder::{ByteOrder, NetworkEndian};

use open::BGPOpen;
use keepalive::BGPKeepalive;
use update::BGPUpdate;
use notification::BGPNotification;

pub const BGP_MAX_MSG_SIZE: usize = 4096;
pub const BGP_HEADER_SIZE: usize = 19;
pub const BGP_OPEN_SIZE: usize = 10;

const BGP_TYPE_OPEN: u8 = 0x01;
const BGP_TYPE_UPDATE: u8 = 0x02;
const BGP_TYPE_NOTIFICATION: u8 = 0x03;
const BGP_TYPE_KEEPALIVE: u8 = 0x04;

#[derive(Debug)]
pub enum BGPMessage {
    Open(BGPOpen),
    Update(BGPUpdate),
    Notification(BGPNotification),
    Keepalive(BGPKeepalive),
}

fn make_bgp_header(length: u16, msg_type: u8) -> [u8; BGP_HEADER_SIZE] {
    let mut buf = [0xFF; BGP_HEADER_SIZE];
    NetworkEndian::write_u16(&mut buf[16..18], BGP_HEADER_SIZE as u16 + length);
    buf[18] = msg_type;
    return buf
}

pub fn message_length(message_buffer: &[u8]) -> usize {
    NetworkEndian::read_u16(&message_buffer[16..18]) as usize
}

impl From<&[u8]> for BGPMessage {
    fn from(buf: &[u8]) -> BGPMessage {
        let (header, rest) = buf.split_at(BGP_HEADER_SIZE);
        let length = message_length(&header);
        let msg_payload = &rest[0..length - BGP_HEADER_SIZE];
        let msg_type = header[18];
        match msg_type {
            BGP_TYPE_OPEN => BGPMessage::Open(msg_payload.into()),
            BGP_TYPE_UPDATE => BGPMessage::Update(msg_payload.into()),
            BGP_TYPE_NOTIFICATION => BGPMessage::Notification(msg_payload.into()),
            BGP_TYPE_KEEPALIVE => BGPMessage::Keepalive(msg_payload.into()),
            _ => unimplemented!("BGP Message type: {:?}", msg_type)
        }
    }
}

impl Into<Vec<u8>> for BGPMessage {
    fn into(self) -> Vec<u8> {
        match self {
            BGPMessage::Open(open) => open.into(),
            BGPMessage::Keepalive(keepalive) => keepalive.into(),
            BGPMessage::Update(update) => update.into(),
            BGPMessage::Notification(_notification) => unimplemented!(),
        }
    }
}

