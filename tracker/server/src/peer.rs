use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use kallax_tracker_primitives::{
    chain_spec::{ChainLayer, ChainMetadata},
    peer::PeerAddress,
};
use kallax_tracker_proto::peer as proto;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::error;

#[derive(Clone, Debug, Default)]
pub struct Service {
    rootchains_peer_addresses: Arc<Mutex<HashMap<String, HashSet<PeerAddress>>>>,
    leafchains_peer_addresses: Arc<Mutex<HashMap<String, HashSet<PeerAddress>>>>,
}

#[tonic::async_trait]
impl proto::PeerService for Service {
    async fn get_peer_addresses(
        &self,
        req: Request<proto::GetPeerAddressesRequest>,
    ) -> Result<Response<proto::GetPeerAddressesResponse>, Status> {
        let chain_metadata = {
            let chain_metadata = req
                .into_inner()
                .chain_metadata
                .ok_or_else(|| error::into_invalid_argument_status("chain_metadata"))?;

            ChainMetadata::try_from(chain_metadata)
                .map_err(|e| Status::invalid_argument(e.to_string()))?
        };

        let addresses = {
            fn map_addresses(addresses: &HashSet<PeerAddress>) -> Vec<proto::PeerAddress> {
                addresses.iter().cloned().map(proto::PeerAddress::from).collect()
            }

            let ChainMetadata { layer: chain_layer, name: chain_name } = &chain_metadata;

            let peer_addresses = match chain_layer {
                ChainLayer::Rootchain => &self.rootchains_peer_addresses,
                ChainLayer::Leafchain => &self.leafchains_peer_addresses,
            };

            peer_addresses.lock().await.get(chain_name).map_or_else(Vec::new, map_addresses)
        };

        Ok(Response::new(proto::GetPeerAddressesResponse {
            chain_metadata: Some(chain_metadata.into()),
            addresses,
        }))
    }

    async fn insert_peer_address(
        &self,
        req: Request<proto::InsertPeerAddressRequest>,
    ) -> Result<Response<proto::InsertPeerAddressResponse>, Status> {
        let remote_addr = req.remote_addr();

        let proto::InsertPeerAddressRequest { chain_metadata, address } = req.into_inner();
        let ChainMetadata { layer: chain_layer, name: chain_name } = {
            let chain_metadata = chain_metadata
                .ok_or_else(|| error::into_invalid_argument_status("chain_metadata"))?;

            ChainMetadata::try_from(chain_metadata)
                .map_err(|e| Status::invalid_argument(e.to_string()))?
        };

        let peer_address = {
            let address = address.ok_or_else(|| error::into_invalid_argument_status("address"))?;
            let mut address = PeerAddress::try_from(address)
                .map_err(|e| Status::invalid_argument(e.to_string()))?;

            if let Some(remote_addr) = remote_addr {
                address.try_make_public(remote_addr);
            }

            address
        };

        let peer_addresses = match chain_layer {
            ChainLayer::Rootchain => &self.rootchains_peer_addresses,
            ChainLayer::Leafchain => &self.leafchains_peer_addresses,
        };

        peer_addresses
            .lock()
            .await
            .entry(chain_name)
            .or_insert_with(HashSet::new)
            .insert(peer_address);

        Ok(Response::new(proto::InsertPeerAddressResponse {}))
    }
}
