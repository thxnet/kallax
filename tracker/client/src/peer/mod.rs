mod error;

use std::collections::HashSet;

use async_trait::async_trait;
use kallax_tracker_primitives::{chain_spec::ChainMetadata, peer::PeerAddress};
use kallax_tracker_proto::peer as proto;
use snafu::ResultExt;

pub use self::error::Error;
use self::error::Result;
use crate::Client;

#[async_trait]
pub trait PeerExt {
    async fn get_peer_addresses(
        &self,
        chain_metadata: &ChainMetadata,
    ) -> Result<HashSet<PeerAddress>>;

    async fn insert_peer_address(
        &self,
        chain_metadata: &ChainMetadata,
        addr: &PeerAddress,
    ) -> Result<()>;
}

#[async_trait]
impl PeerExt for Client {
    async fn get_peer_addresses(
        &self,
        chain_metadata: &ChainMetadata,
    ) -> Result<HashSet<PeerAddress>> {
        proto::PeerServiceClient::new(self.channel.clone())
            .get_peer_addresses(proto::GetPeerAddressesRequest {
                chain_metadata: Some(chain_metadata.clone().into()),
            })
            .await
            .context(error::GetPeerAddressesSnafu)?
            .into_inner()
            .addresses
            .into_iter()
            .map(PeerAddress::try_from)
            .collect::<kallax_tracker_primitives::Result<HashSet<PeerAddress>>>()
            .map_err(Error::from)
    }

    async fn insert_peer_address(
        &self,
        chain_metadata: &ChainMetadata,
        addr: &PeerAddress,
    ) -> Result<()> {
        proto::PeerServiceClient::new(self.channel.clone())
            .insert_peer_address(proto::InsertPeerAddressRequest {
                chain_metadata: Some(chain_metadata.clone().into()),
                address: Some(addr.clone().into()),
            })
            .await
            .context(error::InsertPeerAddressSnafu)?;
        Ok(())
    }
}
