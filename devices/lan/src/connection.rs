use super::network::*;

use core::device::{DeviceMessage, HostMessage};
use core::error;

use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use mio::net::*;
use mio::*;

const DEVICE_AVAILABLE: Token = Token(2);

pub(super) struct TcpConnection {
    socket: TcpStream,
    poll: Poll,
    events: Events,

    received_messages: Vec<DeviceMessage>,
}

impl TcpConnection {
    pub(super) fn new(addr: SocketAddr) -> error::Result<Self> {
        let mut socket = TcpStream::connect(addr)?;
        socket.set_nodelay(true)?;

        let poll = Poll::new()?;
        poll.registry().register(
            &mut socket,
            DEVICE_AVAILABLE,
            Interest::READABLE | Interest::WRITABLE,
        )?;

        let events = Events::with_capacity(128);

        Ok(Self {
            socket,
            poll,
            events,

            received_messages: vec![],
        })
    }

    pub(super) fn socket(&self) -> &TcpStream {
        &self.socket
    }

    pub(super) fn send(
        &mut self,
        data: &HostMessage,
        timeout: Option<Duration>,
        mut retries: Option<usize>,
    ) -> error::Result<()> {
        let packet = NetworkPacket::serialize(data)?;

        loop {
            let _ = self.poll.poll(&mut self.events, timeout);

            for e in self.events.iter() {
                if e.token() != DEVICE_AVAILABLE || !e.is_writable() {
                    continue;
                }

                if self.socket.send_packet(&packet).is_ok() {
                    return Ok(());
                }
            }

            retries.as_mut().map(|retries| *retries -= 1);
            if retries.is_some_and(|retries| retries == 0) {
                return Err(error::Error::Other("All send retries failed".to_string()));
            }
        }
    }

    pub(super) fn recv_to_buf(&mut self, timeout: Option<Duration>) -> error::Result<()> {
        let _ = self.poll.poll(&mut self.events, timeout);

        for e in self.events.iter() {
            if e.token() != DEVICE_AVAILABLE || !e.is_readable() {
                continue;
            }

            loop {
                let packet = self.socket().recv_packet();
                match packet {
                    Ok(packet) => {
                        let Ok(data) = packet.deserialize() else {
                            continue;
                        };
                        self.received_messages.push(data);
                    }
                    Err(error::Error::WrongNetworkPacketHeader) => continue,
                    Err(error::Error::Io(io)) if io.kind() == io::ErrorKind::WouldBlock => break,
                    Err(error::Error::Io(io)) if is_io_error_critical(&io) => {
                        return Err(error::Error::Other("Device disconnected".to_string()));
                    }
                    Err(err) => return Err(err),
                }
            }
        }

        Ok(())
    }

    pub(super) fn filter_received_messages<F>(&mut self, f: F)
    where
        F: FnMut(DeviceMessage) -> Option<DeviceMessage>,
    {
        self.received_messages = self.received_messages.drain(..).filter_map(f).collect();
    }
}

fn is_io_error_critical(err: &io::Error) -> bool {
    match err.kind() {
        io::ErrorKind::ConnectionAborted => true,
        io::ErrorKind::ConnectionReset => true,
        io::ErrorKind::BrokenPipe => true,
        io::ErrorKind::UnexpectedEof => true,
        _ => false,
    }
}
