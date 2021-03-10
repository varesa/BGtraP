use byteorder::{ByteOrder, NetworkEndian};
use std::convert::TryInto;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const BGP_MAX_MSG_SIZE: usize = 4096;
const BGP_HEADER_SIZE: usize = 19;
const BGP_OPEN_SIZE: usize = 10;

const BGP_TYPE_OPEN: u8 = 0x01;
const BGP_TYPE_UPDATE: u8 = 0x02;
const BGP_TYPE_NOTIFICATION: u8 = 0x03;
const BGP_TYPE_KEEPALIVE: u8 = 0x04;

#[derive(Debug)]
struct BGPOpen {
    version: u8,
    sender_as: u16,
    hold_time: u16,
    bgp_id: u32,
    opt_params_len: u8,
    opt_params: ()
}
#[derive(Debug)]
struct BGPUpdate {}
#[derive(Debug)]
struct BGPNotification {
    error_code: u8,
    error_subcode: u8,
    data: Vec<u8>,
}
#[derive(Debug)]
struct BGPKeepalive {}

fn make_bgp_header(length: u16, msg_type: u8) -> [u8; BGP_HEADER_SIZE] {
    let mut buf = [0xFF; BGP_HEADER_SIZE];
    NetworkEndian::write_u16(&mut buf[16..18], BGP_HEADER_SIZE as u16 + length);
    buf[18] = msg_type;
    return buf
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

impl From<&[u8]> for BGPUpdate {
    fn from(buf: &[u8]) -> BGPUpdate {
        BGPUpdate {}
    }
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

impl From<&[u8]> for BGPKeepalive {
    fn from(buf: &[u8]) -> BGPKeepalive {
        BGPKeepalive {}
    }
}

impl Into<[u8; BGP_HEADER_SIZE]> for BGPKeepalive {
    fn into(self) -> [u8; BGP_HEADER_SIZE] {
        make_bgp_header(0, BGP_TYPE_KEEPALIVE)
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
                println!("R: {:?}", &bgp_message);
                match bgp_message {
                    BGPMessage::Open(_) => {
                        let open = BGPOpen {
                            version: 4,
                            sender_as: 65002,
                            hold_time: 60,
                            bgp_id: 1234567890,
                            opt_params_len: 0,
                            opt_params: (),
                        };
                        println!("S: {:?}", &open);
                        let buf: [u8; BGP_HEADER_SIZE + BGP_OPEN_SIZE] = open.into();
                        if let Err(e) = socket.write_all(&buf[..]).await {
                            eprintln!("failed to write to socket, err = {:?}", e);
                            return;
                        }
                    },
                    BGPMessage::Keepalive(_) => {
                        let keepalive = BGPKeepalive {};
                        println!("S: {:?}", &keepalive);
                        let buf: [u8; BGP_HEADER_SIZE] = keepalive.into();
                        if let Err(e) = socket.write_all(&buf[..]).await {
                            eprintln!("failed to write to socket, err = {:?}", e);
                            return;
                        }
                    },
                    _ => {}
                }
                /*if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket, err = {:?}", e);
                    return;
                }*/
            }
        });
    }
}
