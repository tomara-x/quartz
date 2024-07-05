use bevy::prelude::*;
use rosc::{
    decoder::{decode_udp, MTU},
    encoder, OscMessage, OscPacket, OscType,
};
use std::net::UdpSocket;

#[derive(Resource)]
pub struct OscSender {
    pub host: String,
    pub port: u16,
}

impl OscSender {
    pub fn send<T, I>(&self, address: &str, args: T)
    where
        T: IntoIterator<Item = I>,
        I: Into<OscType>,
    {
        if let Ok(client) = UdpSocket::bind("0.0.0.0:0") {
            let packet = OscPacket::Message(OscMessage {
                addr: address.to_string(),
                args: args.into_iter().map(Into::into).collect(),
            });
            let buf = encoder::encode(&packet).unwrap();
            let _ = client.send_to(&buf, format!("{}:{}", self.host, self.port));
        }
    }
}

#[derive(Resource)]
pub struct OscReceiver {
    pub socket: Option<UdpSocket>,
}

impl OscReceiver {
    pub fn init(&mut self, port: u16) {
        if let Ok(socket) = UdpSocket::bind(format!("0.0.0.0:{}", port)) {
            socket.set_nonblocking(true).unwrap();
            self.socket = Some(socket);
        } else {
            warn!("can't bind! another app is using port {}", port);
            self.socket = None;
        }
    }
}

pub fn unpacket(packet: OscPacket, buffer: &mut Vec<OscMessage>) {
    match packet {
        OscPacket::Message(msg) => {
            buffer.push(msg);
        }
        OscPacket::Bundle(bundle) => {
            bundle.content.iter().for_each(|packet| {
                unpacket(packet.clone(), buffer);
            });
        }
    }
}

pub fn receive_packet(socket: &UdpSocket) -> Option<OscPacket> {
    let mut buf = [0u8; MTU];
    if let Ok(num_bytes) = socket.recv(&mut buf) {
        if let Ok((_, packet)) = decode_udp(&buf[0..num_bytes]) {
            return Some(packet);
        }
    }
    None
}
