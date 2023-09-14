use core::{audio_system::audio::MuxedAudioBuffer, error};
use mio::net::*;
use std::{
    collections::VecDeque,
    net::{Ipv4Addr, SocketAddr},
};

use crate::network::UdpSocketExt;

pub(super) struct AudioStream {
    socket: UdpSocket,

    received_audio: VecDeque<MuxedAudioBuffer>,
}

impl AudioStream {
    pub(super) fn new(addr: SocketAddr) -> error::Result<Self> {
        let socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))?;
        socket.connect(addr)?;

        Ok(Self {
            socket,

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
        while let Ok(packet) = self.socket.recv_packet() {
            let muxed_audio = MuxedAudioBuffer(packet.into_bytes());

            self.received_audio.push_back(muxed_audio);
        }
    }

    pub(super) fn pull(&mut self) -> Option<MuxedAudioBuffer> {
        self.received_audio.pop_front()
    }
}
