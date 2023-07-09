use core::error;

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

use mio::net::UdpSocket;

pub(super) struct NetworkPacket(Vec<u8>);

impl NetworkPacket {
    pub(super) const HEADER: [u8; 5] = [0xF, 0xF, 0x0, 0x12, 0xE];
    pub(super) const HEADER_LEN: usize = Self::HEADER.len();
    pub(super) const NO_SIZE_BYTES: usize = usize::BITS as usize / 8;
    
    pub(super) fn serialize<S>(data: &S) -> error::Result<Self>
    where
        S: serde::Serialize,
    {
        let data_ser = serde_json::to_vec(data)?;
        let mut bytes = Self::HEADER.to_vec();
        let size_bytes = data_ser.len().to_be_bytes();

        bytes.extend_from_slice(&size_bytes);
        bytes.extend(data_ser);

        Ok(Self(bytes))
    }

    pub(super) fn deserialize<D>(self) -> error::Result<D>
    where
        D: for<'de> serde::Deserialize<'de>,
    {
        Ok(serde_json::from_slice(
            &self.0[Self::HEADER_LEN + Self::NO_SIZE_BYTES..],
        )?)
    }

    pub(super) fn is_header_correct(header: &[u8]) -> bool {
        header.starts_with(&Self::HEADER)
    }

    pub(super) fn read_size_from_header(header: &[u8]) -> usize {
        let mut size_bytes = [0; Self::NO_SIZE_BYTES];
        size_bytes.clone_from_slice(&header[Self::HEADER_LEN..]);

        usize::from_be_bytes(size_bytes)
    }
}

impl std::convert::AsRef<[u8]> for NetworkPacket {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

pub(super) trait NetworkPacketUdpExt: Sized {
    fn send_to(self, socket: &UdpSocket, addr: SocketAddr) -> error::Result<()>;
    fn recv_from(socket: &UdpSocket) -> error::Result<(Self, SocketAddr)>;
    fn is_pending_from(socket: &UdpSocket) -> bool;
}

impl NetworkPacketUdpExt for NetworkPacket {
    fn send_to(self, socket: &UdpSocket, addr: SocketAddr) -> error::Result<()> {
        socket.send_to(&self.0, addr)?;

        Ok(())
    }

    fn recv_from(socket: &UdpSocket) -> error::Result<(Self, SocketAddr)> {
        let mut header = [0u8; NetworkPacket::HEADER_LEN + NetworkPacket::NO_SIZE_BYTES];
        let header_len = socket.peek(&mut header)?;

        if !Self::is_header_correct(&header) {
            return Err(error::Error::WrongNetworkPacketHeader);
        }

        let size = NetworkPacket::read_size_from_header(&header[..header_len]);
        let mut bytes = vec![0; Self::HEADER_LEN + Self::NO_SIZE_BYTES + size];

        let (_, sender_addr) = socket.recv_from(&mut bytes)?;

        Ok((Self(bytes), sender_addr))
    }

    fn is_pending_from(socket: &UdpSocket) -> bool {
        let mut header = [0u8; NetworkPacket::HEADER.len()];
        socket
            .peek_from(&mut header)
            .is_ok_and(|(n, _)| Self::is_header_correct(&header[..n]))
    }
}

pub(super) trait NetworkPacketTcpExt: Sized {
    fn send(self, socket: &TcpStream) -> error::Result<()>;
    fn recv(socket: &TcpStream) -> error::Result<Self>;
}

impl NetworkPacketTcpExt for NetworkPacket {
    fn send(self, mut socket: &TcpStream) -> error::Result<()> {
        socket.write(&self.0)?;

        Ok(())
    }

    fn recv(mut socket: &TcpStream) -> error::Result<Self> {
        let mut header = [0u8; NetworkPacket::HEADER_LEN + NetworkPacket::NO_SIZE_BYTES];
        let header_len = socket.peek(&mut header)?;

        if !Self::is_header_correct(&header) {
            return Err(error::Error::WrongNetworkPacketHeader);
        }

        let size = NetworkPacket::read_size_from_header(&header[..header_len]);
        let mut bytes = vec![0; Self::HEADER_LEN + Self::NO_SIZE_BYTES + size];

        let _ = socket.read(&mut bytes)?;

        Ok(Self(bytes))
    }
}