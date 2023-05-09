use std::{
    fmt,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
};

use sc_network::multiaddr::Protocol;
use snafu::ResultExt;

use crate::{error, error::Error};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PeerAddress(pub sc_network::Multiaddr);

impl PeerAddress {
    #[must_use]
    pub fn is_lookback(&self) -> bool {
        let is_ipv4_lookback =
            self.0.iter().take(1).all(|component| component == Protocol::Ip4(Ipv4Addr::LOCALHOST));
        let is_ipv6_lookback =
            self.0.iter().take(1).all(|component| component == Protocol::Ip6(Ipv6Addr::LOCALHOST));
        is_ipv4_lookback || is_ipv6_lookback
    }

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

impl fmt::Display for PeerAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
