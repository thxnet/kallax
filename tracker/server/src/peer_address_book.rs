use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};

use kallax_primitives::ExternalEndpoint;
use time::Duration;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PeerAddress {
    address: kallax_primitives::PeerAddress,

    external: Option<ExternalEndpoint>,
}

type PeerAddresses = HashMap<PeerAddress, Option<time::OffsetDateTime>>;

#[derive(Clone, Debug)]
pub struct PeerAddressBook {
    ttl: Duration,

    books: Arc<Mutex<HashMap<String, PeerAddresses>>>,
}

impl Default for PeerAddressBook {
    fn default() -> Self {
        Self::new()
    }
}

impl PeerAddressBook {
    pub fn new() -> Self {
        Self::with_ttl(std::time::Duration::from_secs(120))
    }

    pub fn with_ttl(ttl: std::time::Duration) -> Self {
        let ttl = Duration::new(i64::try_from(ttl.as_secs()).unwrap_or_default(), 0);
        Self { ttl, books: Arc::default() }
    }
}

impl PeerAddressBook {
    pub async fn fetch_peers<ChainId>(
        &self,
        chain_id: ChainId,
    ) -> Vec<kallax_primitives::PeerAddress>
    where
        ChainId: fmt::Display,
    {
        let chain_id = chain_id.to_string();
        self.books.lock().await.get(&chain_id).map_or_else(Vec::new, |addresses| {
            addresses.iter().map(|(addr, _)| addr.address.clone()).collect()
        })
    }

    /// Fetches peer addresses with external endpoint rewriting for cross-cluster
    /// connectivity. Peers with an external endpoint get their address rewritten
    /// via `exposed()`. Peers without an external endpoint (or where `exposed()`
    /// returns `None`) fall back to their original internal address, ensuring they
    /// are never silently dropped from the peer list.
    pub async fn fetch_exposed_peers<ChainId>(
        &self,
        chain_id: ChainId,
    ) -> Vec<kallax_primitives::PeerAddress>
    where
        ChainId: fmt::Display,
    {
        let chain_id = chain_id.to_string();
        self.books.lock().await.get(&chain_id).map_or_else(Vec::new, |addresses| {
            let mut addresses = addresses
                .iter()
                .map(|(PeerAddress { address, external }, _)| {
                    external
                        .as_ref()
                        .and_then(|external_endpoint| {
                            let exposed = address.exposed(external_endpoint);
                            if exposed.is_none() {
                                tracing::warn!(
                                    %address, ?external_endpoint,
                                    "Peer has external endpoint but exposed() returned None; \
                                     falling back to original address"
                                );
                            }
                            exposed
                        })
                        .unwrap_or_else(|| address.clone())
                })
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            addresses.sort_unstable();
            addresses
        })
    }

    #[allow(dead_code)]
    pub async fn insert_reserved<ChainId>(
        &self,
        chain_id: ChainId,
        peer_address: kallax_primitives::PeerAddress,
        external_endpoint: Option<ExternalEndpoint>,
    ) where
        ChainId: fmt::Display,
    {
        let chain_id = chain_id.to_string();
        self.books
            .lock()
            .await
            .entry(chain_id)
            .or_insert_with(HashMap::new)
            .insert(PeerAddress { address: peer_address, external: external_endpoint }, None);
    }

    pub async fn insert<ChainId>(
        &self,
        chain_id: ChainId,
        peer_address: kallax_primitives::PeerAddress,
        external_endpoint: Option<ExternalEndpoint>,
    ) where
        ChainId: fmt::Display,
    {
        let chain_id = chain_id.to_string();
        self.books.lock().await.entry(chain_id).or_insert_with(HashMap::new).insert(
            PeerAddress { address: peer_address, external: external_endpoint },
            Some(time::OffsetDateTime::now_utc()),
        );
    }

    pub async fn flush(&self) {
        tracing::info!("Start to flush stalled peer addresses");

        let now = time::OffsetDateTime::now_utc();

        let mut books = self.books.lock().await;

        for (_, book) in &mut books.iter_mut() {
            book.retain(|PeerAddress { address, .. }, last_update_time| {
                last_update_time.map_or(true, |last_update_time| {
                    if (now - last_update_time) < self.ttl {
                        true
                    } else {
                        tracing::info!("`{address}` is stalled, removing it");
                        false
                    }
                })
            });
        }
        drop(books);
        tracing::info!("Flushing stalled peer addresses completed");
    }

    pub async fn clear(&self) {
        let mut books = self.books.lock().await;
        books.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use kallax_primitives::{ExternalEndpoint, PeerAddress as PrimitivePeerAddress};

    use super::*;

    const PEER_ADDR_WITH_IP: &str =
        "/ip4/10.0.0.1/tcp/30333/p2p/12D3KooWEYdR9WN6tyReBTmngueGTRAQztkWrNLx9kCw9aQ3Tbwo";
    const PEER_ADDR_WITH_DNS: &str =
        "/dns/node.example.com/tcp/30333/p2p/12D3KooWEYdR9WN6tyReBTmngueGTRAQztkWrNLx9kCw9aQ3Tbwo";

    #[tokio::test]
    async fn fetch_exposed_peers_includes_peers_without_external_endpoint() {
        let book = PeerAddressBook::new();
        let addr = PrimitivePeerAddress::from_str(PEER_ADDR_WITH_IP).unwrap();
        book.insert("chain-1", addr.clone(), None).await;

        let peers = book.fetch_exposed_peers("chain-1").await;
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].to_string(), PEER_ADDR_WITH_IP);
    }

    #[tokio::test]
    async fn fetch_exposed_peers_rewrites_when_external_endpoint_present() {
        let book = PeerAddressBook::new();
        let addr = PrimitivePeerAddress::from_str(PEER_ADDR_WITH_IP).unwrap();
        let external = ExternalEndpoint { host: "node.example.com".to_string(), port: 54321 };
        book.insert("chain-1", addr, Some(external)).await;

        let peers = book.fetch_exposed_peers("chain-1").await;
        assert_eq!(peers.len(), 1);
        assert!(
            peers[0].to_string().contains("/dns/node.example.com/tcp/54321/"),
            "Expected rewritten DNS address, got: {}",
            peers[0]
        );
    }

    #[tokio::test]
    async fn fetch_exposed_peers_falls_back_when_exposed_returns_none() {
        let book = PeerAddressBook::new();
        // A /dns/ base address won't match the Ip4/Ip6 check in exposed(), returning None
        let addr = PrimitivePeerAddress::from_str(PEER_ADDR_WITH_DNS).unwrap();
        let external = ExternalEndpoint { host: "other.example.com".to_string(), port: 9999 };
        book.insert("chain-1", addr.clone(), Some(external)).await;

        let peers = book.fetch_exposed_peers("chain-1").await;
        assert_eq!(peers.len(), 1);
        // Should fall back to original /dns/ address, not be dropped
        assert_eq!(peers[0].to_string(), PEER_ADDR_WITH_DNS);
    }
}
