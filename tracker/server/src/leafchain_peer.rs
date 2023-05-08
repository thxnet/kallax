use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use kallax_primitives::PeerAddress;
use kallax_tracker_proto as proto;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::error;

#[derive(Clone, Debug, Default)]
pub struct Service {
    peer_addresses: Arc<Mutex<HashMap<String, HashSet<PeerAddress>>>>,

    allow_loopback_ip: bool,
}

impl Service {
    #[must_use]
    pub fn new(allow_loopback_ip: bool) -> Self {
        Self { peer_addresses: Arc::default(), allow_loopback_ip }
    }
}

#[tonic::async_trait]
impl proto::LeafchainPeerService for Service {
    async fn get(
        &self,
        req: Request<proto::GetLeafchainPeerAddressesRequest>,
    ) -> Result<Response<proto::GetLeafchainPeerAddressesResponse>, Status> {
        let chain_id = req.into_inner().chain_id;

        let addresses = {
            self.peer_addresses.lock().await.get(&chain_id).map_or_else(Vec::new, |addresses| {
                addresses.iter().cloned().map(proto::PeerAddress::from).collect()
            })
        };

        Ok(Response::new(proto::GetLeafchainPeerAddressesResponse { addresses }))
    }

    async fn insert(
        &self,
        req: Request<proto::InsertLeafchainPeerAddressRequest>,
    ) -> Result<Response<proto::InsertLeafchainPeerAddressResponse>, Status> {
        let proto::InsertLeafchainPeerAddressRequest { chain_id, address } = req.into_inner();

        let peer_address = {
            let address = address.ok_or_else(|| error::into_invalid_argument_status("address"))?;
            PeerAddress::try_from(address).map_err(|e| Status::invalid_argument(e.to_string()))?
        };

        if peer_address.is_lookback() && !self.allow_loopback_ip {
            tracing::info!(
                "New peer `{peer_address}` is in loopback network, skip to insert to chain \
                 `{chain_id}`"
            );
            return Ok(Response::new(proto::InsertLeafchainPeerAddressResponse {}));
        }

        tracing::info!("Insert new peer `{peer_address}` to chain `{chain_id}`");

        self.peer_addresses
            .lock()
            .await
            .entry(chain_id)
            .or_insert_with(HashSet::new)
            .insert(peer_address);

        Ok(Response::new(proto::InsertLeafchainPeerAddressResponse {}))
    }
}
