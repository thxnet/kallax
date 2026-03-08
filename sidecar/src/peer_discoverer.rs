use std::{
    collections::{HashMap, HashSet, VecDeque},
    str::FromStr,
    sync::Arc,
};

use kallax_primitives::{BlockchainLayer, ExternalEndpoint, PeerAddress};
use kallax_tracker_grpc_client::{Client as TrackerClient, LeafchainPeer, RootchainPeer};
use serde::Serialize;
use snafu::ResultExt;
use substrate_rpc_client::{
    ws_client as connect_substrate_websocket_endpoint, SystemApi, WsClient,
};
use tokio::sync::Mutex;

use crate::{
    error,
    error::{Error, Result},
};

type Hash = sp_core::H256;
type BlockNumber = u128;

/// Consecutive polling cycles a peer must be absent before removal.
/// With POLLING_INTERVAL=1s and tracker TTL=120s, 60 cycles = 60s grace period.
const STALE_THRESHOLD: u32 = 60;

const ERROR_RING_CAPACITY: usize = 50;

// --- Diagnostic data structures ---

#[derive(Clone, Debug, Default, Serialize)]
pub struct DiagnosticSnapshot {
    pub identity: IdentityInfo,
    pub registration: RegistrationInfo,
    pub discovery_funnel: DiscoveryFunnel,
    pub connections: ConnectionStatus,
    pub stale_counters: HashMap<String, u32>,
    pub health: HealthCounters,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct IdentityInfo {
    pub peer_id: Option<String>,
    pub detected_public_ip: Option<String>,
    pub local_listen_addresses: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct RegistrationInfo {
    pub chain_id: String,
    pub blockchain_layer: String,
    pub external_endpoint: Option<String>,
    pub registered_addresses_count: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct DiscoveryFunnel {
    pub raw_from_tracker: usize,
    pub after_self_filter: usize,
    pub after_stale_filter: usize,
    pub after_known_filter: usize,
    pub after_loopback_filter: usize,
    pub new_peers_added: Vec<String>,
    pub stalled_peers_removed: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ConnectionStatus {
    pub reserved_peers: Vec<String>,
    pub connected_peers: Vec<ConnectedPeerInfo>,
    pub is_syncing: bool,
    pub substrate_peer_count: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ConnectedPeerInfo {
    pub peer_id: String,
    pub best_number: u128,
    pub roles: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct HealthCounters {
    pub last_successful_poll: Option<String>,
    pub last_error: Option<String>,
    pub consecutive_failures: u32,
    pub total_polls: u64,
    pub total_failures: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorEntry {
    pub timestamp: String,
    pub cycle: u64,
    pub category: String,
    pub message: String,
}

// --- Error ring buffer ---

#[derive(Debug)]
pub struct ErrorRing {
    entries: VecDeque<ErrorEntry>,
}

impl ErrorRing {
    pub fn new() -> Self {
        Self { entries: VecDeque::with_capacity(ERROR_RING_CAPACITY) }
    }

    pub fn push(&mut self, entry: ErrorEntry) {
        if self.entries.len() >= ERROR_RING_CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    pub fn entries(&self) -> Vec<ErrorEntry> {
        self.entries.iter().cloned().collect()
    }
}

pub type SharedErrorRing = Arc<Mutex<ErrorRing>>;
pub type SharedDiagnostic = Arc<Mutex<Option<DiagnosticSnapshot>>>;

#[derive(Debug)]
pub struct PeerDiscoverer {
    chain_id: String,

    blockchain_layer: BlockchainLayer,

    substrate_websocket_endpoint: http::Uri,

    tracker_client: TrackerClient,

    substrate_client: Option<WsClient>,

    allow_loopback_ip: bool,

    prefer_exposed_peers: bool,

    external_endpoint: Option<ExternalEndpoint>,

    stale_counters: HashMap<String, u32>,

    diagnostic: SharedDiagnostic,

    detected_public_ip: Option<String>,

    health: HealthCounters,

    error_ring: SharedErrorRing,

    cycle_count: u64,

    cached_peer_id: Option<String>,
}

impl PeerDiscoverer {
    #[inline]
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chain_id: String,
        blockchain_layer: BlockchainLayer,
        substrate_websocket_endpoint: http::Uri,
        tracker_client: TrackerClient,
        allow_loopback_ip: bool,
        prefer_exposed_peers: bool,
        external_endpoint: Option<ExternalEndpoint>,
        diagnostic: SharedDiagnostic,
        detected_public_ip: Option<String>,
        error_ring: SharedErrorRing,
    ) -> Self {
        Self {
            chain_id,
            blockchain_layer,
            substrate_websocket_endpoint,
            tracker_client,
            allow_loopback_ip,
            prefer_exposed_peers,
            substrate_client: None,
            external_endpoint,
            stale_counters: HashMap::new(),
            diagnostic,
            detected_public_ip,
            health: HealthCounters::default(),
            error_ring,
            cycle_count: 0,
            cached_peer_id: None,
        }
    }

    async fn record_error(&self, category: &str, message: &str) {
        let entry = ErrorEntry {
            timestamp: time::OffsetDateTime::now_utc().to_string(),
            cycle: self.cycle_count,
            category: category.to_string(),
            message: message.to_string(),
        };
        self.error_ring.lock().await.push(entry);
    }

    // FIXME: split the function into smaller pieces
    #[allow(clippy::too_many_lines)]
    pub async fn execute(&mut self) -> Result<()> {
        self.cycle_count += 1;
        self.health.total_polls += 1;

        let result = self.execute_inner().await;

        match &result {
            Ok(()) => {
                self.health.last_successful_poll =
                    Some(time::OffsetDateTime::now_utc().to_string());
                self.health.consecutive_failures = 0;
            }
            Err(err) => {
                self.health.consecutive_failures += 1;
                self.health.total_failures += 1;
                let msg = err.to_string();
                self.health.last_error = Some(msg.clone());
                self.record_error("execute", &msg).await;
            }
        }

        result
    }

    #[allow(clippy::too_many_lines)]
    async fn execute_inner(&mut self) -> Result<()> {
        let substrate_client = if let Some(substrate_client) = self.substrate_client.take() {
            substrate_client
        } else {
            connect_substrate_websocket_endpoint(self.substrate_websocket_endpoint.to_string())
                .await
                .map_err(|error| Error::ConnectSubstrateNode {
                    uri: self.substrate_websocket_endpoint.clone(),
                    error,
                })?
        };

        // Cache peer ID on first successful connection
        if self.cached_peer_id.is_none() {
            match SystemApi::<Hash, BlockNumber>::system_local_peer_id(&substrate_client).await {
                Ok(peer_id) => {
                    tracing::debug!("Cached local peer ID: {peer_id}");
                    self.cached_peer_id = Some(peer_id);
                }
                Err(err) => {
                    tracing::warn!("Failed to fetch local peer ID: {err}");
                    self.record_error("substrate_rpc", &format!("system_local_peer_id: {err}"))
                        .await;
                }
            }
        }

        // fetch listen addresses from local peer
        let listen_addresses: HashSet<_> =
            SystemApi::<Hash, BlockNumber>::system_local_listen_addresses(&substrate_client)
                .await
                .context(error::FetchLocalListenAddressesFromSubstrateNodeSnafu)?
                .into_iter()
                .map(|addr| PeerAddress::from_str(addr.as_str()))
                .collect::<std::result::Result<HashSet<_>, kallax_primitives::Error>>()
                .unwrap_or_else(|err| {
                    tracing::error!("Error occurs while parsing peer address, error: {err}");
                    HashSet::default()
                });
        tracing::debug!(
            "Local addresses get from local Substrate-based node: {listen_addresses:?}"
        );

        // fetch peer addresses from local node
        let current_reserved_peers =
            match SystemApi::<Hash, BlockNumber>::system_reserved_peers(&substrate_client).await {
                Ok(peers) => {
                    tracing::debug!(
                        "Current peers that local Substrate-based node connected: {peers:?}"
                    );
                    peers
                }
                Err(err) => {
                    tracing::error!("{err}");
                    self.record_error("substrate_rpc", &format!("system_reserved_peers: {err}"))
                        .await;
                    Vec::new()
                }
            };
        tracing::debug!("Current reserved peers: {current_reserved_peers:?}");

        // fetch new peer addresses from tracker
        let mut potential_new_peers = {
            let blockchain_layer = self.blockchain_layer;

            match blockchain_layer {
                BlockchainLayer::Rootchain => RootchainPeer::get(
                    &self.tracker_client,
                    &self.chain_id,
                    self.prefer_exposed_peers,
                )
                .await
                .map_err(|err| {
                    tracing::error!("{err}");
                    err
                })
                .unwrap_or_default(),
                BlockchainLayer::Leafchain => LeafchainPeer::get(
                    &self.tracker_client,
                    &self.chain_id,
                    self.prefer_exposed_peers,
                )
                .await
                .map_err(|err| {
                    tracing::error!("{err}");
                    err
                })
                .unwrap_or_default(),
            }
        };
        let raw_from_tracker = potential_new_peers.len();
        tracing::debug!("Peers advertised from tracker: {potential_new_peers:?}");

        let stalled_peers = {
            let tracker_peer_ids: HashSet<String> =
                potential_new_peers.iter().map(PeerAddress::id).collect();
            detect_stalled_peers(
                &mut self.stale_counters,
                &current_reserved_peers,
                &tracker_peer_ids,
            )
        };

        // filter out new peer addresses with funnel tracking
        let (new_peers, funnel) = {
            // remove local node addresses (compare by peer ID, not full multiaddr,
            // because exposed addresses have different IP/port)
            filter_self_addresses(&mut potential_new_peers, &listen_addresses);
            let after_self_filter = potential_new_peers.len();

            // remove stalled peer addresses
            potential_new_peers.retain(|addr| !stalled_peers.contains(&addr.id()));
            let after_stale_filter = potential_new_peers.len();

            // remove known peers
            let mut to_remove: HashSet<PeerAddress> = HashSet::new();
            for peer_id in &current_reserved_peers {
                to_remove.extend(
                    potential_new_peers
                        .iter()
                        .filter(|addr| addr.to_string().contains(peer_id.as_str()))
                        .map(Clone::clone),
                );
            }

            potential_new_peers.retain(|addr| !to_remove.contains(addr));
            let after_known_filter = potential_new_peers.len();

            let filtered = if self.allow_loopback_ip {
                potential_new_peers
            } else {
                potential_new_peers
                    .into_iter()
                    .filter_map(|addr| if addr.is_loopback() { None } else { Some(addr) })
                    .collect()
            };
            let after_loopback_filter: HashSet<PeerAddress> = filtered;

            let new_peer_addrs: Vec<String> =
                after_loopback_filter.iter().map(ToString::to_string).collect();

            let funnel = DiscoveryFunnel {
                raw_from_tracker,
                after_self_filter,
                after_stale_filter,
                after_known_filter,
                after_loopback_filter: after_loopback_filter.len(),
                new_peers_added: new_peer_addrs,
                stalled_peers_removed: stalled_peers.clone(),
            };

            (after_loopback_filter, funnel)
        };

        // add new peer addresses into local node
        if new_peers.is_empty() {
            tracing::debug!("No new peer will be advertised to local Substrate-based node");
        } else {
            tracing::info!(
                "New peers that will be advertised to local Substrate-based node: {new_peers:?}"
            );
            let add_reserved_peers_futs = new_peers.into_iter().map(|addr| {
                SystemApi::<Hash, BlockNumber>::system_add_reserved_peer(
                    &substrate_client,
                    addr.to_string(),
                )
            });

            if let Err(err) = futures::future::try_join_all(add_reserved_peers_futs).await {
                tracing::error!(
                    "Error occurs while advertising new peers to Substrate-based node, error: \
                     {err}"
                );
                self.record_error("substrate_rpc", &format!("system_add_reserved_peer: {err}"))
                    .await;
            }
        }

        // remove stalled peer addresses in local node
        if stalled_peers.is_empty() {
            tracing::debug!("No stalled peer will be removed from local Substrate-based node");
        } else {
            tracing::info!(
                "Stalled peers are removing from local Substrate-based node: {stalled_peers:?}"
            );
            let remove_reserved_peers_futs = stalled_peers.into_iter().map(|addr| {
                SystemApi::<Hash, BlockNumber>::system_remove_reserved_peer(&substrate_client, addr)
            });

            if let Err(err) = futures::future::try_join_all(remove_reserved_peers_futs).await {
                tracing::error!(
                    "Error occurs while removing stalled peers from Substrate-based node, error: \
                     {err}"
                );
                self.record_error("substrate_rpc", &format!("system_remove_reserved_peer: {err}"))
                    .await;
            }
        }

        // advertise local address via tracker
        tracing::info!("Advertise local address via tracker");
        let registered_addresses_count = listen_addresses.len();
        let res = {
            let blockchain_layer = self.blockchain_layer;
            match blockchain_layer {
                BlockchainLayer::Rootchain => {
                    futures::future::try_join_all(listen_addresses.iter().map(|local_address| {
                        RootchainPeer::insert(
                            &self.tracker_client,
                            &self.chain_id,
                            local_address,
                            &self.external_endpoint,
                        )
                    }))
                    .await
                    .map_err(|e| e.to_string())
                }
                BlockchainLayer::Leafchain => {
                    futures::future::try_join_all(listen_addresses.iter().map(|local_address| {
                        LeafchainPeer::insert(
                            &self.tracker_client,
                            &self.chain_id,
                            local_address,
                            &self.external_endpoint,
                        )
                    }))
                    .await
                    .map_err(|e| e.to_string())
                }
            }
        };

        if let Err(err) = res {
            tracing::error!("Error occurs while advertising peers to Tracker, error: {err}");
            self.record_error("tracker_register", &err).await;
        }

        // Fetch connection info from Substrate RPC (best-effort)
        let connections = {
            let mut conn = ConnectionStatus {
                reserved_peers: current_reserved_peers.clone(),
                ..ConnectionStatus::default()
            };

            match SystemApi::<Hash, BlockNumber>::system_peers(&substrate_client).await {
                Ok(peers) => {
                    conn.connected_peers = peers
                        .iter()
                        .map(|p| ConnectedPeerInfo {
                            peer_id: p.peer_id.clone(),
                            best_number: p.best_number,
                            roles: p.roles.to_string(),
                        })
                        .collect();
                    conn.substrate_peer_count = peers.len();
                }
                Err(err) => {
                    tracing::debug!("Failed to fetch system_peers: {err}");
                }
            }

            match SystemApi::<Hash, BlockNumber>::system_health(&substrate_client).await {
                Ok(health) => {
                    conn.is_syncing = health.is_syncing;
                }
                Err(err) => {
                    tracing::debug!("Failed to fetch system_health: {err}");
                }
            }

            conn
        };

        // update diagnostic snapshot
        *self.diagnostic.lock().await = Some(DiagnosticSnapshot {
            identity: IdentityInfo {
                peer_id: self.cached_peer_id.clone(),
                detected_public_ip: self.detected_public_ip.clone(),
                local_listen_addresses: listen_addresses.iter().map(ToString::to_string).collect(),
            },
            registration: RegistrationInfo {
                chain_id: self.chain_id.clone(),
                blockchain_layer: format!("{}", self.blockchain_layer),
                external_endpoint: self.external_endpoint.as_ref().map(ToString::to_string),
                registered_addresses_count,
            },
            discovery_funnel: funnel,
            connections,
            stale_counters: self.stale_counters.clone(),
            health: self.health.clone(),
        });

        self.substrate_client = Some(substrate_client);

        Ok(())
    }
}

/// Filters out addresses belonging to the local node by comparing peer IDs.
/// Uses peer ID (not full multiaddr) because exposed addresses have different IP/port.
fn filter_self_addresses(
    potential_new_peers: &mut HashSet<PeerAddress>,
    listen_addresses: &HashSet<PeerAddress>,
) {
    let local_peer_ids: HashSet<String> =
        listen_addresses.iter().map(PeerAddress::id).filter(|id| !id.is_empty()).collect();
    potential_new_peers.retain(|addr| {
        let id = addr.id();
        id.is_empty() || !local_peer_ids.contains(&id)
    });
}

fn detect_stalled_peers(
    stale_counters: &mut HashMap<String, u32>,
    current_reserved_peers: &[String],
    tracker_peer_ids: &HashSet<String>,
) -> Vec<String> {
    let mut stalled = Vec::new();
    for peer in current_reserved_peers {
        if tracker_peer_ids.contains(peer) {
            stale_counters.remove(peer);
        } else {
            let count = stale_counters.entry(peer.clone()).or_insert(0);
            *count += 1;
            if *count >= STALE_THRESHOLD {
                tracing::info!(peer = %peer, absent_cycles = *count, "Peer exceeded stale threshold");
                stalled.push(peer.clone());
            } else {
                tracing::debug!(
                    peer = %peer, absent_cycles = *count, threshold = STALE_THRESHOLD,
                    "Peer absent but within grace period"
                );
            }
        }
    }
    // Clean up counters for peers no longer in reserved list
    let reserved_set: HashSet<&String> = current_reserved_peers.iter().collect();
    stale_counters.retain(|p, _| reserved_set.contains(p));
    stalled
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use kallax_primitives::PeerAddress;

    use super::*;

    const PEER_ID: &str = "12D3KooWEYdR9WN6tyReBTmngueGTRAQztkWrNLx9kCw9aQ3Tbwo";

    fn make_ip4_addr(peer_id: &str) -> PeerAddress {
        PeerAddress::from_str(&format!("/ip4/10.0.0.1/tcp/30333/p2p/{peer_id}")).unwrap()
    }

    fn make_dns_addr(peer_id: &str) -> PeerAddress {
        PeerAddress::from_str(&format!("/dns/node.example.com/tcp/54321/p2p/{peer_id}")).unwrap()
    }

    // Bug 1 tests

    #[test]
    fn filter_self_addresses_removes_own_peer_id() {
        let local = make_ip4_addr(PEER_ID);
        let exposed = make_dns_addr(PEER_ID);
        let other = make_ip4_addr("12D3KooWHdiAxVd8uMQR1hGWXccidmfCwLqcMpGwR6QcTP6QRMuD");

        let listen_addresses: HashSet<PeerAddress> = [local].into_iter().collect();
        let mut potential = [exposed, other.clone()].into_iter().collect();

        filter_self_addresses(&mut potential, &listen_addresses);

        assert_eq!(potential.len(), 1);
        assert!(potential.contains(&other));
    }

    #[test]
    fn filter_self_addresses_keeps_addresses_without_peer_id() {
        let local = make_ip4_addr(PEER_ID);
        let no_p2p = PeerAddress::from_str("/ip4/10.0.0.2/tcp/30333").unwrap();

        let listen_addresses: HashSet<PeerAddress> = [local].into_iter().collect();
        let mut potential = [no_p2p.clone()].into_iter().collect();

        filter_self_addresses(&mut potential, &listen_addresses);

        assert_eq!(potential.len(), 1);
        assert!(potential.contains(&no_p2p));
    }

    // Bug 3 tests

    #[test]
    fn detect_stalled_peers_grace_period() {
        let mut counters = HashMap::new();
        let reserved = vec!["peer-A".to_string()];
        let tracker_ids: HashSet<String> = HashSet::new(); // peer-A absent from tracker

        // Absent for 59 cycles → not stalled
        for _ in 0..59 {
            let stalled = detect_stalled_peers(&mut counters, &reserved, &tracker_ids);
            assert!(stalled.is_empty(), "Should not be stalled before threshold");
        }

        // 60th cycle → stalled
        let stalled = detect_stalled_peers(&mut counters, &reserved, &tracker_ids);
        assert_eq!(stalled, vec!["peer-A".to_string()]);
    }

    #[test]
    fn detect_stalled_peers_resets_counter_on_return() {
        let mut counters = HashMap::new();
        let reserved = vec!["peer-A".to_string()];
        let empty_tracker: HashSet<String> = HashSet::new();

        // Accumulate 5 absent cycles
        for _ in 0..5 {
            detect_stalled_peers(&mut counters, &reserved, &empty_tracker);
        }
        assert_eq!(counters.get("peer-A"), Some(&5));

        // Peer reappears in tracker
        let tracker_with_peer: HashSet<String> = ["peer-A".to_string()].into_iter().collect();
        let stalled = detect_stalled_peers(&mut counters, &reserved, &tracker_with_peer);
        assert!(stalled.is_empty());
        assert!(!counters.contains_key("peer-A"));
    }

    #[test]
    fn detect_stalled_peers_cleans_up_removed_peers() {
        let mut counters = HashMap::new();
        counters.insert("old-peer".to_string(), 10);

        // old-peer is no longer in reserved list
        let reserved = vec!["new-peer".to_string()];
        let tracker_ids: HashSet<String> = HashSet::new();
        detect_stalled_peers(&mut counters, &reserved, &tracker_ids);

        // old-peer counter should be cleaned up
        assert!(!counters.contains_key("old-peer"));
        // new-peer should have counter of 1
        assert_eq!(counters.get("new-peer"), Some(&1));
    }

    #[test]
    fn error_ring_respects_capacity() {
        let mut ring = ErrorRing::new();
        for i in 0..(ERROR_RING_CAPACITY + 10) {
            ring.push(ErrorEntry {
                timestamp: String::new(),
                cycle: i as u64,
                category: "test".to_string(),
                message: format!("error {i}"),
            });
        }
        let entries = ring.entries();
        assert_eq!(entries.len(), ERROR_RING_CAPACITY);
        // oldest should have been evicted
        assert_eq!(entries[0].cycle, 10);
    }
}
