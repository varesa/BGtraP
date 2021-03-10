use byteorder::{ByteOrder, NetworkEndian};
use std::convert::TryInto;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const BGP_MAX_MSG_SIZE: usize = 4096;
const BGP_HEADER_SIZE: usize = 19;

enum BGPMessageType {
    OPEN,
    UPDATE,
    NOTIFICATION,
    KEEPALIVE,
    UNKNOWN,
}

impl From<u8> for BGPMessageType {
    fn from(value: u8) -> BGPMessageType {
        match value {
            0x01 => BGPMessageType::OPEN,
            0x02 => BGPMessageType::UPDATE,
            0x03 => BGPMessageType::NOTIFICATION,
            0x04 => BGPMessageType::KEEPALIVE,
            _ => BGPMessageType::UNKNOWN
        }
    }
}

struct BGPOpen {
    
}

impl From<&[u8]> for BGPOpen {
    fn from(buf: &[u8]) -> BGPOpen {
        BGPOpen {

        }
    }
}

struct BGPUpdate {

}

enum BGPMessageContent {
    Open(BGPOpen),
    Update(BGPUpdate),
}

struct BGPMessage {
    header: [u8; 19],
    content: BGPMessageContent,
}

impl From<[u8; BGP_MAX_MSG_SIZE]> for BGPMessage {
    fn from(buf: [u8; BGP_MAX_MSG_SIZE]) -> BGPMessage {
        let (header, rest) = buf.split_at(BGP_HEADER_SIZE);
        let size = NetworkEndian::read_u16(&header[16..18]);
        let msg_type = header[18];
        let content = match msg_type.into() {
            BGPMessageType::OPEN => BGPMessageContent::Open(BGPOpen::from(rest)),
            _ => unimplemented!("BGP Message type: {:?}", msg_type)
        };
        let msg = BGPMessage {
            header: header.try_into().expect("slice with incorrect length"),
            content: content.into()
        };
        msg
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
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket, err = {:?}", e);
                    return;
                }
            }
        });
    }
}
