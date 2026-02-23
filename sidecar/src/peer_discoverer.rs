use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use kallax_primitives::{BlockchainLayer, ExternalEndpoint, PeerAddress};
use kallax_tracker_grpc_client::{Client as TrackerClient, LeafchainPeer, RootchainPeer};
use snafu::ResultExt;
use substrate_rpc_client::{
    ws_client as connect_substrate_websocket_endpoint, SystemApi, WsClient,
};

use crate::{
    error,
    error::{Error, Result},
};

type Hash = sp_core::H256;
type BlockNumber = u128;

/// Consecutive polling cycles a peer must be absent before removal.
/// With POLLING_INTERVAL=1s and tracker TTL=120s, 60 cycles = 60s grace period.
const STALE_THRESHOLD: u32 = 60;

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
}

impl PeerDiscoverer {
    #[inline]
    #[must_use]
    pub fn new(
        chain_id: String,
        blockchain_layer: BlockchainLayer,
        substrate_websocket_endpoint: http::Uri,
        tracker_client: TrackerClient,
        allow_loopback_ip: bool,
        prefer_exposed_peers: bool,
        external_endpoint: Option<ExternalEndpoint>,
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
        }
    }

    // FIXME: split the function into smaller pieces
    #[allow(clippy::too_many_lines)]
    pub async fn execute(&mut self) -> Result<()> {
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
                .map_err(|err| tracing::error!("{err}"))
                .unwrap_or_default(),
                BlockchainLayer::Leafchain => LeafchainPeer::get(
                    &self.tracker_client,
                    &self.chain_id,
                    self.prefer_exposed_peers,
                )
                .await
                .map_err(|err| tracing::error!("{err}"))
                .unwrap_or_default(),
            }
        };
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

        // filter out new peer addresses
        let new_peers = {
            // remove local node addresses (compare by peer ID, not full multiaddr,
            // because exposed addresses have different IP/port)
            filter_self_addresses(&mut potential_new_peers, &listen_addresses);

            // remove stalled peer addresses
            potential_new_peers.retain(|addr| !stalled_peers.contains(&addr.id()));

            // remove known peers
            let mut to_remove: HashSet<PeerAddress> = HashSet::new();
            for peer_id in current_reserved_peers {
                to_remove.extend(
                    potential_new_peers
                        .iter()
                        .filter(|addr| addr.to_string().contains(&peer_id))
                        .map(Clone::clone),
                );
            }

            potential_new_peers.retain(|addr| !to_remove.contains(addr));

            if self.allow_loopback_ip {
                potential_new_peers
            } else {
                potential_new_peers
                    .into_iter()
                    .filter_map(|addr| if addr.is_loopback() { None } else { Some(addr) })
                    .collect()
            }
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
            }
        }

        // advertise local address via tracker
        tracing::info!("Advertise local address via tracker");
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
        }

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
}
