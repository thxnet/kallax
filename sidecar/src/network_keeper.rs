use std::{collections::HashSet, str::FromStr};

use kallax_tracker_client::{Client as TrackerClient, PeerExt};
use kallax_tracker_primitives::{chain_spec::ChainMetadata, peer::PeerAddress};
use snafu::ResultExt;
use substrate_rpc_client::{ws_client as connect_substrate_websocket_endpoint, SystemApi};

use crate::{
    error,
    error::{Error, Result},
};

type Hash = ();
type BlockNumber = u128;

#[derive(Debug)]
pub struct NetworkKeeper {
    chain_metadata: ChainMetadata,

    substrate_websocket_endpoint: http::Uri,

    tracker_client: TrackerClient,
}

impl NetworkKeeper {
    #[inline]
    #[must_use]
    pub fn new(
        chain_metadata: ChainMetadata,
        substrate_websocket_endpoint: http::Uri,
        tracker_client: TrackerClient,
    ) -> Self {
        Self { chain_metadata, substrate_websocket_endpoint, tracker_client }
    }

    pub async fn execute(&self) -> Result<()> {
        let substrate_client =
            connect_substrate_websocket_endpoint(self.substrate_websocket_endpoint.to_string())
                .await
                .map_err(|error| Error::ConnectSubstrateNode {
                    uri: self.substrate_websocket_endpoint.clone(),
                    error,
                })?;

        // fetch listen addresses from local peer
        let listen_addresses: HashSet<_> =
            SystemApi::<Hash, BlockNumber>::system_local_listen_addresses(&substrate_client)
                .await
                .context(error::FetchLocalListenAddressesFromSubstrateNodeSnafu)?
                .into_iter()
                .map(|addr| PeerAddress::from_str(addr.as_str()))
                .collect::<std::result::Result<HashSet<_>, kallax_tracker_primitives::Error>>()
                .unwrap_or_else(|err| {
                    tracing::error!("Error occurs while parsing peer address, error: {err}");
                    HashSet::default()
                });
        tracing::debug!(
            "Local addresses get from local Substrate-based node: {listen_addresses:?}"
        );

        // fetch peer addresses from local node
        let current_peers = SystemApi::<Hash, BlockNumber>::system_peers(&substrate_client)
            .await
            .context(error::FetchPeersFromSubstrateNodeSnafu)?;
        tracing::debug!(
            "Current peers that local Substrate-based node connected: {current_peers:?}"
        );

        // fetch new peer addresses from tracker
        let potential_new_peers = self
            .tracker_client
            .get_peer_addresses(&self.chain_metadata)
            .await
            .context(error::GetPeerAddressesFromTrackerSnafu)?;

        // filter out new peer addresses
        let new_peers = potential_new_peers;
        // TODO:

        // add new peer addresses into local node
        tracing::info!(
            "New peers that will be advertised to local Substrate-based node: {new_peers:?}"
        );
        let add_reserved_peers_futs: Vec<_> = new_peers
            .into_iter()
            .map(|addr| {
                SystemApi::<Hash, BlockNumber>::system_add_reserved_peer(
                    &substrate_client,
                    addr.to_string(),
                )
            })
            .collect();
        if let Err(err) = futures::future::try_join_all(add_reserved_peers_futs).await {
            tracing::error!(
                "Error occurs while advertising new peers to Substrate-based node, error: {err}"
            )
        }

        // advertise local address via tracker
        tracing::info!("Advertise local address via tracker");
        let advertising_futs = listen_addresses.iter().map(|local_address| {
            self.tracker_client.insert_peer_address(&self.chain_metadata, local_address)
        });
        if let Err(err) = futures::future::try_join_all(advertising_futs).await {
            tracing::error!("Error occurs while advertising peers to Tracker, error: {err}")
        }

        Ok(())
    }
}
