use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};

use kallax_primitives::ExternalEndpoint;
use serde::Serialize;
use time::Duration;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PeerAddress {
    address: kallax_primitives::PeerAddress,

    external: Option<ExternalEndpoint>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiagnosticPeer {
    pub address: String,
    pub external_endpoint: Option<ExternalEndpoint>,
    pub last_seen: Option<String>,
    pub is_reserved: bool,
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

    /// Fetches both internal and exposed addresses for every peer, enabling
    /// topology-agnostic peer discovery. Peers with an external endpoint will
    /// have both their original (internal) address and the exposed (external)
    /// address included. This lets libp2p connect via whichever route works.
    pub async fn fetch_all_peers<ChainId>(
        &self,
        chain_id: ChainId,
    ) -> Vec<kallax_primitives::PeerAddress>
    where
        ChainId: fmt::Display,
    {
        let chain_id = chain_id.to_string();
        self.books.lock().await.get(&chain_id).map_or_else(Vec::new, |addresses| {
            let mut result = HashSet::new();
            for PeerAddress { address, external } in addresses.keys() {
                result.insert(address.clone()); // always include internal
                if let Some(ep) = external.as_ref() {
                    if let Some(exposed) = address.exposed(ep) {
                        result.insert(exposed);
                    } else {
                        tracing::warn!(%address, external_endpoint = ?ep, "exposed() returned None");
                    }
                }
            }
            let mut result: Vec<_> = result.into_iter().collect();
            result.sort_unstable();
            result
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

    pub async fn peer_counts(&self) -> HashMap<String, usize> {
        let books = self.books.lock().await;
        books.iter().map(|(chain_id, addresses)| (chain_id.clone(), addresses.len())).collect()
    }

    #[allow(dead_code)]
    pub async fn diagnostic_snapshot(&self) -> HashMap<String, Vec<DiagnosticPeer>> {
        let books = self.books.lock().await;
        books
            .iter()
            .map(|(chain_id, addresses)| {
                let peers = addresses
                    .iter()
                    .map(|(peer, last_seen)| DiagnosticPeer {
                        address: peer.address.to_string(),
                        external_endpoint: peer.external.clone(),
                        last_seen: last_seen.map(|t| t.to_string()),
                        is_reserved: last_seen.is_none(),
                    })
                    .collect();
                (chain_id.clone(), peers)
            })
            .collect()
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

    #[tokio::test]
    async fn fetch_all_peers_returns_both_when_external_present() {
        let book = PeerAddressBook::new();
        let addr = PrimitivePeerAddress::from_str(PEER_ADDR_WITH_IP).unwrap();
        let external = ExternalEndpoint { host: "node.example.com".to_string(), port: 54321 };
        book.insert("chain-1", addr, Some(external)).await;

        let peers = book.fetch_all_peers("chain-1").await;
        // Should contain both the internal /ip4/ address and the exposed /dns/ address
        assert_eq!(peers.len(), 2);
        let addrs: Vec<String> = peers.iter().map(ToString::to_string).collect();
        assert!(addrs.iter().any(|a| a.contains("/ip4/10.0.0.1/")), "missing internal address");
        assert!(
            addrs.iter().any(|a| a.contains("/dns/node.example.com/tcp/54321/")),
            "missing exposed address"
        );
    }

    #[tokio::test]
    async fn fetch_all_peers_returns_only_internal_when_no_external() {
        let book = PeerAddressBook::new();
        let addr = PrimitivePeerAddress::from_str(PEER_ADDR_WITH_IP).unwrap();
        book.insert("chain-1", addr, None).await;

        let peers = book.fetch_all_peers("chain-1").await;
        assert_eq!(peers.len(), 1);
        assert!(peers[0].to_string().contains("/ip4/10.0.0.1/"));
    }

    #[tokio::test]
    async fn fetch_all_peers_keeps_internal_when_exposed_returns_none() {
        let book = PeerAddressBook::new();
        // A /dns/ base address won't match the Ip4/Ip6 check in exposed(), returning None
        let addr = PrimitivePeerAddress::from_str(PEER_ADDR_WITH_DNS).unwrap();
        let external = ExternalEndpoint { host: "other.example.com".to_string(), port: 9999 };
        book.insert("chain-1", addr.clone(), Some(external)).await;

        let peers = book.fetch_all_peers("chain-1").await;
        // exposed() returns None for /dns/ base → only internal address kept
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].to_string(), PEER_ADDR_WITH_DNS);
    }

    #[tokio::test]
    async fn fetch_all_peers_deduplicates() {
        let book = PeerAddressBook::new();
        let addr1 = PrimitivePeerAddress::from_str(PEER_ADDR_WITH_IP).unwrap();
        let addr2 = PrimitivePeerAddress::from_str(
            "/ip4/10.0.0.2/tcp/30333/p2p/12D3KooWHdiAxVd8uMQR1hGWXccidmfCwLqcMpGwR6QcTP6QRMuD",
        )
        .unwrap();
        let external = ExternalEndpoint { host: "node.example.com".to_string(), port: 54321 };

        book.insert("chain-1", addr1, Some(external.clone())).await;
        book.insert("chain-1", addr2, Some(external)).await;

        let peers = book.fetch_all_peers("chain-1").await;
        // 2 internal + 2 exposed = 4 unique addresses (different peer IDs)
        assert_eq!(peers.len(), 4);
        // Verify no duplicates
        let unique: HashSet<String> = peers.iter().map(ToString::to_string).collect();
        assert_eq!(unique.len(), peers.len());
    }
}
