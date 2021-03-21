mod bgp;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use bgp::{BGP_MAX_MSG_SIZE, BGPMessage, message_length};
use bgp::open::BGPOpen;
use bgp::keepalive::BGPKeepalive;
use bgp::errors::BgpError;

const LOG_MESSAGES: bool = true;

async fn send_message(message: BGPMessage, socket: &mut TcpStream) -> Result<(), BgpError>{
    if LOG_MESSAGES { println!("S: {:#?}", &message); }
    let buf: Vec<u8> = message.into();
    socket.write_all(&buf[..]).await?;
    Ok(())
}

async fn handle_message(message: &BGPMessage, socket: &mut TcpStream) -> Result<(), BgpError> {
    if LOG_MESSAGES { println!("R: {:#?}", &message); }
    match message {
        BGPMessage::Open(_) => {
            let open = BGPOpen {
                version: 4,
                sender_as: 65002,
                hold_time: 30,
                bgp_id: 1234567890,
                opt_params_len: 0,
                opt_params: (),
            };
            send_message(BGPMessage::Open(open), socket).await?;
        },
        BGPMessage::Keepalive(_) => {
            let keepalive = BGPKeepalive {};
            send_message(BGPMessage::Keepalive(keepalive), socket).await?;
        },
        _ => {}
    }
    Ok(())
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
                let mut bytes_left = n;
                let mut i = 0;
                while bytes_left > 0 {
                    let bgp_message_buf = &buf[i..n];
                    let bgp_message_length = message_length(&bgp_message_buf);
                    let bgp_message: BGPMessage = bgp_message_buf.into();
                    handle_message(&bgp_message, &mut socket).await.expect("Failed to handle message");
                    i += bgp_message_length;
                    bytes_left -= bgp_message_length;
                }
            }
        });
    }
}
