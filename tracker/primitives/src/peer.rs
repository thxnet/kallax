use std::{fmt, net::SocketAddr, str::FromStr};

use kallax_tracker_proto::peer as proto;
use sc_network::multiaddr::Protocol;
use snafu::ResultExt;

use crate::{error, error::Error};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PeerAddress(sc_network::Multiaddr);

impl PeerAddress {
    pub fn try_make_public(&mut self, socket_addr: SocketAddr) {
        if socket_addr.ip().is_loopback() {
            return;
        }

        let new_addr = self.0.replace(0, |protocol| {
            if matches!(protocol, Protocol::Ip4(..) | Protocol::Ip6(..)) {
                match socket_addr {
                    SocketAddr::V4(ipv4) => Some(Protocol::Ip4(*ipv4.ip())),
                    SocketAddr::V6(ipv6) => Some(Protocol::Ip6(*ipv6.ip())),
                }
            } else {
                None
            }
        });

        if let Some(a) = new_addr {
            self.0 = a;
        }
    }
}

impl FromStr for PeerAddress {
    type Err = Error;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        sc_network::Multiaddr::from_str(address)
            .map(Self)
            .context(error::InvalidPeerAddressSnafu { value: address })
    }
}

impl TryFrom<proto::PeerAddress> for PeerAddress {
    type Error = Error;

    fn try_from(proto::PeerAddress { address }: proto::PeerAddress) -> Result<Self, Self::Error> {
        Self::from_str(&address)
    }
}

impl From<PeerAddress> for proto::PeerAddress {
    fn from(PeerAddress(address): PeerAddress) -> Self { Self { address: address.to_string() } }
}

impl fmt::Display for PeerAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
