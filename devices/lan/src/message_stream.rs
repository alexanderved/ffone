use super::network::*;

use super::{DeviceMessage, HostMessage};
use core::error;

use std::collections::VecDeque;
use std::io;
use std::net::SocketAddr;

use mio::net::*;

pub(super) struct MessageStream {
    socket: TcpStream,

    pub sent_messages: VecDeque<HostMessage>,
    pub received_messages: VecDeque<DeviceMessage>,
}

impl MessageStream {
    pub(super) fn new(addr: SocketAddr) -> error::Result<Self> {
        let socket = TcpStream::connect(addr)?;
        socket.set_nodelay(true)?;

        Ok(Self {
            socket,

            sent_messages: VecDeque::new(),
            received_messages: VecDeque::new(),
        })
    }

    pub(super) fn socket_mut(&mut self) -> &mut TcpStream {
        &mut self.socket
    }

    pub(super) fn send_from_buf(&mut self) -> error::Result<()> {
        while let Some(host_msg) = self.sent_messages.pop_front() {
            let Ok(packet) = NetworkPacket::serialize(&host_msg) else {
                continue;
            };

            match self.socket.write_packet(&packet) {
                Ok(()) => continue,
                Err(error::Error::Io(err)) if err.kind() == io::ErrorKind::WouldBlock => {
                    self.sent_messages.push_front(host_msg);
                    break;
                }
                Err(error::Error::Io(err)) if err.kind() == io::ErrorKind::Interrupted => {
                    self.sent_messages.push_front(host_msg);
                    continue;
                }
                Err(error::Error::Io(err)) if is_io_error_critical(&err) => {
                    return Err(error::Error::DeviceUnlinked);
                }
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }

    pub(super) fn recv_to_buf(&mut self) {
        loop {
            let packet = match self.socket.read_packet() {
                Ok(packet) => packet,
                Err(error::Error::Io(err)) if err.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            };

            match packet.deserialize() {
                Ok(device_msg) => self.received_messages.push_back(device_msg),
                Err(_) => continue,
            }
        }
    }

    pub(super) fn push(&mut self, host_msg: HostMessage) {
        self.sent_messages.push_back(host_msg);
    }

    pub(super) fn pull(&mut self) -> Option<DeviceMessage> {
        self.received_messages.pop_front()
    }
}
