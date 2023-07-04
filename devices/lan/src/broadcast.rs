use super::*;

use core::error;

use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use mio::net::UdpSocket;
use mio::{Events, Interest, Poll, Token};

const IDENTITY_RECEIVED: Token = Token(1);
const SOCKET_TIMEOUT: Duration = Duration::from_micros(0);

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

impl From<(IpAddr, IdentityPacket)> for LanDeviceInfo {
    fn from((ip_addr, net_packet): (IpAddr, IdentityPacket)) -> Self {
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
    bytes: Vec<u8>,
}

impl BroadcastListener {
    pub(super) fn new(port: u16) -> error::Result<Self> {
        let mut socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))?;

        let poll = Poll::new()?;
        poll.registry()
            .register(&mut socket, IDENTITY_RECEIVED, Interest::READABLE)?;

        let events = Events::with_capacity(128);
        let bytes = vec![0; 128];

        Ok(Self {
            socket,
            poll,

            events,
            bytes,
        })
    }

    pub(super) fn recv(&mut self) -> error::Result<impl Iterator<Item = LanDeviceInfo> + 'static> {
        let mut lan_infos = HashSet::new();
        self.poll.poll(&mut self.events, Some(SOCKET_TIMEOUT))?;

        'events: for e in self.events.iter() {
            if e.token() != IDENTITY_RECEIVED {
                continue;
            }

            while udp_socket_has_pending_datagram(&self.socket) {
                let Ok((len, sender_addr)) =
                    self.socket.recv_from(&mut self.bytes) else {
                        continue 'events;
                    };
                let Ok(identity_packet) =
                    serde_json::from_slice::<IdentityPacket>(&self.bytes[..len]) else {
                        continue;
                    };

                lan_infos.insert((identity_packet, sender_addr.ip()).into());
            }
        }

        Ok(lan_infos.into_iter())
    }
}

fn udp_socket_has_pending_datagram(socket: &UdpSocket) -> bool {
    socket.peek_from(&mut [0]).is_ok()
}
