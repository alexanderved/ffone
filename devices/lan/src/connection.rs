use super::network::*;

use core::error;

use std::net::SocketAddr;

use mio::net::*;
use mio::*;

const DEVICE_AVAILABLE: Token = Token(2);

pub(super) struct TcpConnection {
    socket: TcpStream,
    poll: Poll,
    events: Events,
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
        })
    }

    pub(super) fn socket(&self) -> &TcpStream {
        &self.socket
    }

    pub(super) fn send<S>(&mut self, data: &S) -> error::Result<()>
    where
        S: serde::Serialize,
    {
        if !self.is_open() {
            return Err(error::Error::DeviceDisconnected);
        }

        let packet = NetworkPacket::serialize(data)?;

        loop {
            self.poll.poll(&mut self.events, None)?;

            for e in self.events.iter() {
                if e.is_write_closed() {
                    return Err(error::Error::DeviceDisconnected);
                }

                if e.token() != DEVICE_AVAILABLE || !e.is_writable() {
                    continue;
                }

                if self.socket.send_packet(&packet).is_ok() {
                    return Ok(());
                }
            }
        }
    }

    pub(super) fn recv<D>(&mut self) -> error::Result<D>
    where
        D: for<'de> serde::Deserialize<'de>,
    {
        if !self.is_open() {
            return Err(error::Error::DeviceDisconnected);
        }

        loop {
            self.poll.poll(&mut self.events, None)?;

            for e in self.events.iter() {
                if e.is_read_closed() {
                    return Err(error::Error::DeviceDisconnected);
                }

                if e.token() != DEVICE_AVAILABLE || !e.is_readable() {
                    continue;
                }

                let Ok(packet) = self.socket.recv_packet() else {
                    continue;
                };
                let data = packet.deserialize();

                if data.is_ok() {
                    return data;
                }
            }
        }
    }

    pub(super) fn is_open(&self) -> bool {
        use std::io;

        let res = self.socket.peek(&mut [0]);
        //dbg!(&res);

        match res {
            Ok(0) => false,
            Err(err) if err.kind() == io::ErrorKind::ConnectionAborted => false,
            Err(err) if err.kind() == io::ErrorKind::ConnectionReset => false,
            Err(err) if err.kind() == io::ErrorKind::BrokenPipe => false,
            Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => false,
            _ => true,
        }
    }
}
