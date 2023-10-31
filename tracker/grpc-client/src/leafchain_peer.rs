use std::{collections::HashSet, fmt};

use async_trait::async_trait;
use kallax_primitives::{ExternalEndpoint, PeerAddress};
use kallax_tracker_proto as proto;

use crate::{
    error::{
        ClearLeafchainPeerAddressError, GetLeafchainPeerAddressError,
        InsertLeafchainPeerAddressError,
    },
    Client,
};

#[async_trait]
pub trait LeafchainPeer {
    async fn get<S>(
        &self,
        chain_name: S,
    ) -> Result<HashSet<PeerAddress>, GetLeafchainPeerAddressError>
    where
        S: fmt::Display + Send + Sync;

    async fn insert<S>(
        &self,
        chain_name: S,
        addr: &PeerAddress,
        external_endpoint: &Option<ExternalEndpoint>,
    ) -> Result<(), InsertLeafchainPeerAddressError>
    where
        S: fmt::Display + Send + Sync;

    async fn clear(&self) -> Result<(), ClearLeafchainPeerAddressError>;
}

#[async_trait]
impl LeafchainPeer for Client {
    async fn get<S>(
        &self,
        chain_id: S,
    ) -> Result<HashSet<PeerAddress>, GetLeafchainPeerAddressError>
    where
        S: fmt::Display + Send + Sync,
    {
        proto::LeafchainPeerServiceClient::new(self.channel.clone())
            .get(proto::GetLeafchainPeerAddressesRequest { chain_id: chain_id.to_string() })
            .await
            .map_err(|source| GetLeafchainPeerAddressError::Status { source })?
            .into_inner()
            .addresses
            .into_iter()
            .map(PeerAddress::try_from)
            .collect::<Result<HashSet<PeerAddress>, _>>()
            .map_err(GetLeafchainPeerAddressError::from)
    }

    async fn insert<S>(
        &self,
        chain_id: S,
        addr: &PeerAddress,
        external_endpoint: &Option<ExternalEndpoint>,
    ) -> Result<(), InsertLeafchainPeerAddressError>
    where
        S: fmt::Display + Send + Sync,
    {
        proto::LeafchainPeerServiceClient::new(self.channel.clone())
            .insert(proto::InsertLeafchainPeerAddressRequest {
                chain_id: chain_id.to_string(),
                address: Some(addr.clone().into()),
                external_endpoint: external_endpoint.clone().map(proto::ExternalEndpoint::from),
            })
            .await
            .map_err(|source| InsertLeafchainPeerAddressError::Status { source })?;

        Ok(())
    }

    async fn clear(&self) -> Result<(), ClearLeafchainPeerAddressError> {
        proto::LeafchainPeerServiceClient::new(self.channel.clone())
            .clear(())
            .await
            .map_err(|source| ClearLeafchainPeerAddressError::Status { source })?;

        Ok(())
    }
}
