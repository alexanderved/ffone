use super::network::*;
use super::*;

use core::error;

use std::collections::HashSet;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use mio::net::UdpSocket;
use mio::{Events, Interest, Poll, Token};

const IDENTITY_RECEIVABLE: Token = Token(1);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct IdentityPacket {
    pub(super) name: String,
    pub(super) msg_port: u16,
    pub(super) audio_port: u16,
}

impl From<(IdentityPacket, IpAddr)> for LanDeviceInfo {
    fn from((net_packet, ip_addr): (IdentityPacket, IpAddr)) -> Self {
        Self {
            info: DeviceInfo {
                name: net_packet.name,
            },
            msg_addr: SocketAddr::new(ip_addr, net_packet.msg_port),
            audio_addr: SocketAddr::new(ip_addr, net_packet.audio_port),
        }
    }
}

pub(super) struct UdpBroadcastListener {
    socket: UdpSocket,
    poll: Poll,
    events: Events,
}

impl UdpBroadcastListener {
    pub(super) fn new(port: u16) -> error::Result<Self> {
        let mut socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))?;

        let poll = Poll::new()?;
        poll.registry()
            .register(&mut socket, IDENTITY_RECEIVABLE, Interest::READABLE)?;

        let events = Events::with_capacity(128);

        Ok(Self {
            socket,
            poll,
            events,
        })
    }

    pub(super) fn recv(&mut self) -> error::Result<impl Iterator<Item = LanDeviceInfo> + 'static> {
        let mut lan_infos = HashSet::new();
        self.poll
            .poll(&mut self.events, Some(Duration::from_millis(0)))?;

        for e in self.events.iter() {
            if e.token() != IDENTITY_RECEIVABLE {
                continue;
            }

            loop {
                match recv_device_info(&self.socket) {
                    Ok(lan_info) => {
                        lan_infos.insert(lan_info);
                    }
                    Err(error::Error::Io(err)) if err.kind() == io::ErrorKind::WouldBlock => {
                        break;
                    }
                    Err(_) => continue,
                }
            }
        }

        Ok(lan_infos.into_iter())
    }
}

fn recv_device_info(socket: &UdpSocket) -> error::Result<LanDeviceInfo> {
    let (packet, sender_addr) = socket.recv_packet_from()?;

    let identity = packet.deserialize::<IdentityPacket>()?;
    let info = LanDeviceInfo::from((identity, sender_addr.ip()));

    Ok(info)
}
