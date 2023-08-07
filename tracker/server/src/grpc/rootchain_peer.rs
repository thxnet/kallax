use kallax_primitives::{ExternalEndpoint, PeerAddress};
use kallax_tracker_proto as proto;
use tonic::{Request, Response, Status};

use crate::{error, peer_address_book::PeerAddressBook};

#[derive(Clone, Debug, Default)]
pub struct Service {
    allow_loopback_ip: bool,

    peer_address_book: PeerAddressBook,
}

impl Service {
    #[must_use]
    pub const fn new(allow_loopback_ip: bool, peer_address_book: PeerAddressBook) -> Self {
        Self { allow_loopback_ip, peer_address_book }
    }
}

#[tonic::async_trait]
impl proto::RootchainPeerService for Service {
    async fn get(
        &self,
        req: Request<proto::GetRootchainPeerAddressesRequest>,
    ) -> Result<Response<proto::GetRootchainPeerAddressesResponse>, Status> {
        let chain_id = req.into_inner().chain_id;

        let addresses = self
            .peer_address_book
            .fetch_peers(&chain_id)
            .await
            .into_iter()
            .map(proto::PeerAddress::from)
            .collect();

        Ok(Response::new(proto::GetRootchainPeerAddressesResponse { addresses }))
    }

    async fn insert(
        &self,
        req: Request<proto::InsertRootchainPeerAddressRequest>,
    ) -> Result<Response<proto::InsertRootchainPeerAddressResponse>, Status> {
        let proto::InsertRootchainPeerAddressRequest { chain_id, address, external_endpoint } =
            req.into_inner();

        let peer_address = {
            let address = address.ok_or_else(|| error::into_invalid_argument_status("address"))?;
            PeerAddress::try_from(address).map_err(|e| Status::invalid_argument(e.to_string()))?
        };

        if peer_address.is_loopback() && !self.allow_loopback_ip {
            tracing::info!(
                "New peer `{peer_address}` is in loopback network, skip to insert to chain \
                 `{chain_id}`"
            );

            return Ok(Response::new(proto::InsertRootchainPeerAddressResponse {}));
        }

        tracing::info!("Insert new peer `{peer_address}` to chain `{chain_id}`");

        self.peer_address_book
            .insert(
                chain_id,
                peer_address,
                external_endpoint.and_then(|p| ExternalEndpoint::try_from(p).ok()),
            )
            .await;

        Ok(Response::new(proto::InsertRootchainPeerAddressResponse {}))
    }

    async fn clear(&self, _req: Request<()>) -> Result<Response<()>, Status> {
        self.peer_address_book.clear().await;
        Ok(Response::new(()))
    }
}
