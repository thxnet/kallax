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
}

#[tonic::async_trait]
impl proto::RootchainPeerService for Service {
    async fn get(
        &self,
        req: Request<proto::GetRootchainPeerAddressesRequest>,
    ) -> Result<Response<proto::GetRootchainPeerAddressesResponse>, Status> {
        let chain_id = req.into_inner().chain_id;

        let addresses = {
            self.peer_addresses.lock().await.get(&chain_id).map_or_else(Vec::new, |addresses| {
                addresses.iter().cloned().map(proto::PeerAddress::from).collect()
            })
        };

        Ok(Response::new(proto::GetRootchainPeerAddressesResponse { addresses }))
    }

    async fn insert(
        &self,
        req: Request<proto::InsertRootchainPeerAddressRequest>,
    ) -> Result<Response<proto::InsertRootchainPeerAddressResponse>, Status> {
        let remote_addr = req.remote_addr();

        let proto::InsertRootchainPeerAddressRequest { chain_id, address } = req.into_inner();

        let peer_address = {
            let address = address.ok_or_else(|| error::into_invalid_argument_status("address"))?;
            let mut address = PeerAddress::try_from(address)
                .map_err(|e| Status::invalid_argument(e.to_string()))?;

            if let Some(remote_addr) = remote_addr {
                address.try_make_public(remote_addr);
            }

            address
        };

        self.peer_addresses
            .lock()
            .await
            .entry(chain_id)
            .or_insert_with(HashSet::new)
            .insert(peer_address);

        Ok(Response::new(proto::InsertRootchainPeerAddressResponse {}))
    }
}
