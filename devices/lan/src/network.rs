use core::error;

use std::{
    io::{self, Read, Write},
    net::SocketAddr,
};

#[derive(Debug)]
pub(super) struct NetworkPacket(Vec<u8>);

impl NetworkPacket {
    pub(super) const HEADER_PREFIX: [u8; 5] = [0xF, 0xF, 0x0, 0x12, 0xE];

    pub(super) const HEADER_PREFIX_LEN: usize = Self::HEADER_PREFIX.len();
    pub(super) const NO_SIZE_BYTES: usize = u64::BITS as usize / 8;

    pub(super) const HEADER_LEN: usize = Self::HEADER_PREFIX_LEN + Self::NO_SIZE_BYTES;

    pub(super) fn serialize<S>(data: &S) -> error::Result<Self>
    where
        S: serde::Serialize,
    {
        let data_ser = serde_json::to_vec(data)?;
        let mut bytes = Self::HEADER_PREFIX.to_vec();
        let size_bytes = (data_ser.len() as u64).to_be_bytes();

        bytes.extend_from_slice(&size_bytes);
        bytes.extend(data_ser);

        Ok(Self(bytes))
    }

    pub(super) fn deserialize<D>(self) -> error::Result<D>
    where
        D: for<'de> serde::Deserialize<'de>,
    {
        Ok(serde_json::from_slice(&self.bytes()[Self::HEADER_LEN..])?)
    }

    pub(super) fn is_header_correct(header: &[u8]) -> bool {
        header.starts_with(&Self::HEADER_PREFIX)
    }

    pub(super) fn read_size_from_header(header: &[u8]) -> usize {
        let mut size_bytes = [0; Self::NO_SIZE_BYTES];
        size_bytes.clone_from_slice(&header[Self::HEADER_PREFIX_LEN..]);

        u64::from_be_bytes(size_bytes) as usize
    }

    pub(super) fn bytes(&self) -> &[u8] {
        &self.0
    }

    pub(super) fn len(&self) -> usize {
        self.0.len()
    }
}

impl std::convert::AsRef<[u8]> for NetworkPacket {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

pub(super) trait UdpSocketExt {
    fn send_packet_to(&self, addr: SocketAddr, packet: &NetworkPacket) -> error::Result<usize>;
    fn recv_packet_from(&self) -> error::Result<(NetworkPacket, SocketAddr)>;
    fn has_pending_packet_from(&self) -> bool;
}

impl UdpSocketExt for mio::net::UdpSocket {
    fn send_packet_to(&self, addr: SocketAddr, packet: &NetworkPacket) -> error::Result<usize> {
        Ok(self.send_to(&packet.0, addr)?)
    }

    fn recv_packet_from(&self) -> error::Result<(NetworkPacket, SocketAddr)> {
        let mut header = [0u8; NetworkPacket::HEADER_LEN];
        let (header_len, _) = self.peek_from(&mut header)?;

        if !NetworkPacket::is_header_correct(&header) {
            let _ = self.recv_from(&mut [0]);
            return Err(error::Error::WrongNetworkPacketHeader);
        }

        let size = NetworkPacket::read_size_from_header(&header[..header_len]);
        let mut bytes = vec![0; NetworkPacket::HEADER_LEN + size];

        let (_, sender_addr) = self.recv_from(&mut bytes)?;

        Ok((NetworkPacket(bytes), sender_addr))
    }

    fn has_pending_packet_from(&self) -> bool {
        let mut header = [0u8; NetworkPacket::HEADER_PREFIX_LEN];
        self.peek_from(&mut header)
            .is_ok_and(|(n, _)| NetworkPacket::is_header_correct(&header[..n]))
    }
}

impl UdpSocketExt for std::net::UdpSocket {
    fn send_packet_to(&self, addr: SocketAddr, packet: &NetworkPacket) -> error::Result<usize> {
        Ok(self.send_to(&packet.0, addr)?)
    }

    fn recv_packet_from(&self) -> error::Result<(NetworkPacket, SocketAddr)> {
        let mut header = [0u8; NetworkPacket::HEADER_LEN];
        let header_len = self.peek(&mut header)?;

        if !NetworkPacket::is_header_correct(&header) {
            let _ = self.recv_from(&mut [0]);
            return Err(error::Error::WrongNetworkPacketHeader);
        }

        let size = NetworkPacket::read_size_from_header(&header[..header_len]);
        let mut bytes = vec![0; NetworkPacket::HEADER_LEN + size];

        let (_, sender_addr) = self.recv_from(&mut bytes)?;

        Ok((NetworkPacket(bytes), sender_addr))
    }

    fn has_pending_packet_from(&self) -> bool {
        let mut header = [0u8; NetworkPacket::HEADER_PREFIX_LEN];
        self.peek_from(&mut header)
            .is_ok_and(|(n, _)| NetworkPacket::is_header_correct(&header[..n]))
    }
}

pub(super) trait ReadNetworkPacket: Read {
    fn read_packet(&mut self) -> error::Result<NetworkPacket> {
        let mut header = [0u8; NetworkPacket::HEADER_LEN];
        read_data(&mut *self, &mut header)?;

        if !NetworkPacket::is_header_correct(&header) {
            return Err(error::Error::WrongNetworkPacketHeader);
        }

        let size = NetworkPacket::read_size_from_header(&header);

        let mut bytes = vec![0; NetworkPacket::HEADER_LEN + size];
        bytes[..NetworkPacket::HEADER_LEN].clone_from_slice(&header);

        read_data(self, &mut bytes[NetworkPacket::HEADER_LEN..])?;

        Ok(NetworkPacket(bytes))
    }
}

impl<R: Read> ReadNetworkPacket for R {}

pub(super) trait WriteNetworkPacket: Write {
    fn write_packet(&mut self, packet: &NetworkPacket) -> error::Result<()> {
        let mut bytes_written = 0;
        while bytes_written < packet.len() {
            let n = match self.write(&packet.0) {
                Ok(n) => n,
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => continue,
                Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
                Err(err) if is_io_error_critical(&err) => return Err(error::Error::DeviceUnlinked),
                Err(err) => return Err(err.into()),
            };

            bytes_written += n;
        }
        
        Ok(())
    }
}

impl<W: Write> WriteNetworkPacket for W {}

pub(super) fn is_io_error_critical(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::BrokenPipe
            | io::ErrorKind::UnexpectedEof
    )
}

fn read_data<R: Read>(mut r: R, slice: &mut [u8]) -> error::Result<()> {
    let mut started_reading = false;
    let mut len = 0;
    while len < slice.len() {
        let n = match r.read(&mut slice[len..]) {
            Ok(0) => return Err(error::Error::DeviceUnlinked),
            Ok(n) => n,
            Err(err) if err.kind() == io::ErrorKind::WouldBlock && started_reading => continue,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) if is_io_error_critical(&err) => return Err(error::Error::DeviceUnlinked),
            Err(err) => return Err(err.into()),
        };

        started_reading = true;
        len += n;
    }

    Ok(())
}
