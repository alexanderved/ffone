use super::network::*;
use super::*;

use core::error;

use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use mio::net::UdpSocket;
use mio::{Events, Interest, Poll, Token};

const IDENTITY_RECEIVED: Token = Token(1);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct IdentityPacket {
    pub(super) name: String,
    pub(super) port: u16,
}

impl From<(IdentityPacket, IpAddr)> for LanDeviceInfo {
    fn from((net_packet, ip_addr): (IdentityPacket, IpAddr)) -> Self {
        Self {
            info: DeviceInfo {
                name: net_packet.name,
            },
            addr: SocketAddr::new(ip_addr, net_packet.port),
        }
    }
}

pub(super) struct BroadcastListener {
    socket: UdpSocket,
    poll: Poll,
    events: Events,
}

impl BroadcastListener {
    pub(super) fn new(port: u16) -> error::Result<Self> {
        let mut socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))?;

        let poll = Poll::new()?;
        poll.registry()
            .register(&mut socket, IDENTITY_RECEIVED, Interest::READABLE)?;

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
            .poll(&mut self.events, Some(Duration::from_micros(0)))?;

        for e in self.events.iter() {
            if e.token() != IDENTITY_RECEIVED {
                continue;
            }

            while NetworkPacket::is_pending_from(&self.socket) {
                let Ok(lan_info) = recv_device_info(&self.socket) else {
                    continue;
                };

                lan_infos.insert(lan_info);
            }
        }

        Ok(lan_infos.into_iter())
    }
}

fn recv_device_info(socket: &UdpSocket) -> error::Result<LanDeviceInfo> {
    let (packet, sender_addr) = NetworkPacket::recv_from(socket)?;

    let identity = packet.deserialize::<IdentityPacket>()?;
    let info = LanDeviceInfo::from((identity, sender_addr.ip()));

    Ok(info)
}
