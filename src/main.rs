use byteorder::{ByteOrder, NetworkEndian};
use std::convert::TryInto;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const BGP_MAX_MSG_SIZE: usize = 4096;
const BGP_HEADER_SIZE: usize = 19;

const BGP_TYPE_OPEN: u8 = 0x01;
const BGP_TYPE_UPDATE: u8 = 0x02;
const BGP_TYPE_NOTIFICATION: u8 = 0x03;
const BGP_TYPE_KEEPALIVE: u8 = 0x04;

#[derive(Debug)]
struct BGPOpen {}
#[derive(Debug)]
struct BGPUpdate {}
#[derive(Debug)]
struct BGPNotification {}
#[derive(Debug)]
struct BGPKeepalive {}

impl From<&[u8]> for BGPOpen {
    fn from(buf: &[u8]) -> BGPOpen {
        BGPOpen {}
    }
}

impl From<&[u8]> for BGPUpdate {
    fn from(buf: &[u8]) -> BGPUpdate {
        BGPUpdate {}
    }
}

impl From<&[u8]> for BGPNotification {
    fn from(buf: &[u8]) -> BGPNotification {
        BGPNotification {}
    }
}

impl From<&[u8]> for BGPKeepalive {
    fn from(buf: &[u8]) -> BGPKeepalive {
        BGPKeepalive {}
    }
}

#[derive(Debug)]
enum BGPMessage {
    Open(BGPOpen),
    Update(BGPUpdate),
    Notification(BGPNotification),
    Keepalive(BGPKeepalive),
}

impl From<&[u8]> for BGPMessage {
    fn from(buf: &[u8]) -> BGPMessage {
        let (header, rest) = buf.split_at(BGP_HEADER_SIZE);
        let length = NetworkEndian::read_u16(&header[16..18]) as usize;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:179").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; BGP_MAX_MSG_SIZE];
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => { eprintln!("failed to read from socket, err = {:?}", e); return; }
                };
                let bgp_message: BGPMessage = buf[0..n].into();
                println!("{:?}", &bgp_message);
                /*if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket, err = {:?}", e);
                    return;
                }*/
            }
        });
    }
}
