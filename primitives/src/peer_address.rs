use std::{
    fmt,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
};

use sc_network::multiaddr::Protocol;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{error, error::Error, ExternalEndpoint};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PeerAddress(pub sc_network::Multiaddr);

impl PeerAddress {
    #[must_use]
    pub fn is_loopback(&self) -> bool {
        let is_ipv4_loopback =
            self.0.iter().take(1).all(|component| component == Protocol::Ip4(Ipv4Addr::LOCALHOST));
        let is_ipv6_loopback =
            self.0.iter().take(1).all(|component| component == Protocol::Ip6(Ipv6Addr::LOCALHOST));
        is_ipv4_loopback || is_ipv6_loopback
    }

    #[must_use]
    pub fn exposed(&self, ExternalEndpoint { host, port }: &ExternalEndpoint) -> Option<Self> {
        let new_addr = self
            .0
            .replace(0, |protocol| {
                if matches!(protocol, Protocol::Ip4(..) | Protocol::Ip6(..)) {
                    Some(Protocol::Dns(host.to_string().into()))
                } else {
                    None
                }
            })?
            .replace(1, |protocol| {
                if matches!(protocol, Protocol::Tcp(..)) {
                    Some(Protocol::Tcp(*port))
                } else {
                    None
                }
            })?;

        Some(Self(new_addr))
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

    #[must_use]
    pub fn id(&self) -> String {
        for protocol in self.0.iter() {
            if let Protocol::P2p(id) = protocol {
                return multibase::Base::Base58Btc.encode(id.to_bytes());
            }
        }

        String::new()
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{ExternalEndpoint, PeerAddress};

    #[test]
    fn test_exposed() {
        let addr = PeerAddress::from_str(
            "/ip4/127.0.0.1/tcp/50001/p2p/12D3KooWEYdR9WN6tyReBTmngueGTRAQztkWrNLx9kCw9aQ3Tbwo",
        )
        .unwrap();
        let exposed = addr.exposed(&ExternalEndpoint {
            host: "node.testnet.thxnet.org".to_string(),
            port: 54321,
        });
        let expected = "/dns/node.testnet.thxnet.org/tcp/54321/p2p/\
                        12D3KooWEYdR9WN6tyReBTmngueGTRAQztkWrNLx9kCw9aQ3Tbwo";
        assert_eq!(expected, exposed.unwrap().to_string());
    }
}
