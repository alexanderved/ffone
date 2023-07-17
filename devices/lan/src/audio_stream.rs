use core::{audio_system::EncodedAudioBuffer, error};
use mio::net::*;
use std::{
    collections::VecDeque,
    net::{Ipv4Addr, SocketAddr},
};

pub(super) struct AudioStream {
    socket: UdpSocket,

    bytes: Vec<u8>,
    received_audio: VecDeque<EncodedAudioBuffer>,
}

impl AudioStream {
    pub(super) fn new(addr: SocketAddr) -> error::Result<Self> {
        let socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))?;
        socket.connect(addr)?;

        Ok(Self {
            socket,

            bytes: vec![0; 65536],
            received_audio: VecDeque::new(),
        })
    }

    pub(super) fn socket(&self) -> &UdpSocket {
        &self.socket
    }

    pub(super) fn socket_mut(&mut self) -> &mut UdpSocket {
        &mut self.socket
    }

    pub(super) fn recv_to_buf(&mut self) {
        while let Ok(n) = self.socket.recv(&mut self.bytes) {
            let encoded_audio = EncodedAudioBuffer(self.bytes[..n].to_vec());

            self.received_audio.push_back(encoded_audio);
        }
    }

    pub(super) fn load(&mut self) -> Option<EncodedAudioBuffer> {
        self.received_audio.pop_front()
    }
}
