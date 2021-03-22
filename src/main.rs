mod bgp;

#[macro_use]
extern crate num_derive;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use bgp::{BGP_MAX_MSG_SIZE, BGPMessage, message_length};
use bgp::open::BGPOpen;
use bgp::keepalive::BGPKeepalive;
use bgp::utils::prefix::Prefix;
use bgp::errors::BgpError;
use crate::bgp::update::BGPUpdate;
use crate::bgp::utils::path_attribute::{PathAttribute, AttributeType, AttributeFlag};

const LOG_MESSAGES: bool = true;

macro_rules! log_message_content {
    ($prefix:expr, $message:expr, [$($type:ident),+]) => {
        match $message {
            $(
                BGPMessage::$type(content) => println!("{}: {:#?}", $prefix, &content),
            )+
        }
    }
}

fn log_message(prefix: &str, message: &BGPMessage) {
    if !LOG_MESSAGES {
        return
    }
    log_message_content!(prefix, message, [Open, Keepalive, Update, Notification]);
}

async fn send_message(message: BGPMessage, socket: &mut TcpStream) -> Result<(), BgpError>{
    log_message("S", &message);
    let buf: Vec<u8> = message.into();
    socket.write_all(&buf[..]).await?;
    Ok(())
}

static mut ADV_SENT: bool = false;

async fn demo(socket: &mut TcpStream) -> Result<(), BgpError> {
    unsafe {
        if ADV_SENT {
            return Ok(())
        }
        ADV_SENT = true;
    }

    let advertisement = BGPUpdate {
        withdrawn_routes: vec![],
        network_layer_reachability_information: vec![Prefix { length: 32, prefix: [10, 10, 100, 200]}],
        path_attributes: vec![
            PathAttribute { type_code: AttributeType::Origin, value: vec![2], flags: vec![AttributeFlag::Transitive]},
            PathAttribute { type_code: AttributeType::ASPath, value: vec![], flags: vec![AttributeFlag::ExtendedLength, AttributeFlag::Transitive]},
            PathAttribute { type_code: AttributeType::NextHop, value: vec![192, 168, 10, 5], flags: vec![AttributeFlag::Transitive]},
            PathAttribute { type_code: AttributeType::LocalPref, value: vec![0, 0, 0, 100], flags: vec![AttributeFlag::Transitive]}
        ]
    };
    send_message(BGPMessage::Update(advertisement), socket).await?;
    Ok(())
}

async fn handle_message(message: &BGPMessage, socket: &mut TcpStream) -> Result<(), BgpError> {
    log_message("R", &message);
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
            demo(socket).await?;
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
