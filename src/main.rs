mod bgp;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use bgp::{BGP_HEADER_SIZE, BGP_OPEN_SIZE, BGP_MAX_MSG_SIZE, BGPMessage, message_length};
use bgp::open::BGPOpen;
use bgp::keepalive::BGPKeepalive;

async fn handle_message(message: &BGPMessage, socket: &mut TcpStream) -> () {
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
            //println!("S: {:?}", &open);
            let buf: [u8; BGP_HEADER_SIZE + BGP_OPEN_SIZE] = open.into();
            if let Err(e) = socket.write_all(&buf[..]).await {
                eprintln!("failed to write to socket, err = {:?}", e);
                return;
            }
        },
        BGPMessage::Keepalive(_) => {
            let keepalive = BGPKeepalive {};
            //println!("S: {:?}", &keepalive);
            let buf: [u8; BGP_HEADER_SIZE] = keepalive.into();
            if let Err(e) = socket.write_all(&buf[..]).await {
                eprintln!("failed to write to socket, err = {:?}", e);
                return;
            }
        },
        _ => {}
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
                let mut bytes_left = n;
                let mut i = 0;
                while bytes_left > 0 {
                    let bgp_message_buf = &buf[i..n];
                    let bgp_message_length = message_length(&bgp_message_buf);
                    let bgp_message: BGPMessage = bgp_message_buf.into();
                    handle_message(&bgp_message, &mut socket).await;
                    i += bgp_message_length;
                    bytes_left -= bgp_message_length;
                }

                //println!("R: {:#?}", &bgp_message);
            }
        });
    }
}
