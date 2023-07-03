use std::{collections::HashSet, fmt};

use async_trait::async_trait;
use kallax_primitives::PeerAddress;
use kallax_tracker_proto as proto;

use crate::{
    error::{
        ClearRootchainPeerAddressError, GetRootchainPeerAddressError,
        InsertRootchainPeerAddressError,
    },
    Client,
};

#[async_trait]
pub trait RootchainPeer {
    async fn get<S>(
        &self,
        chain_id: S,
    ) -> Result<HashSet<PeerAddress>, GetRootchainPeerAddressError>
    where
        S: fmt::Display + Send + Sync;

    async fn insert<S>(
        &self,
        chain_id: S,
        addr: &PeerAddress,
    ) -> Result<(), InsertRootchainPeerAddressError>
    where
        S: fmt::Display + Send + Sync;

    async fn clear(&self) -> Result<(), ClearRootchainPeerAddressError>;
}

#[async_trait]
impl RootchainPeer for Client {
    async fn get<S>(
        &self,
        chain_id: S,
    ) -> Result<HashSet<PeerAddress>, GetRootchainPeerAddressError>
    where
        S: fmt::Display + Send + Sync,
    {
        proto::RootchainPeerServiceClient::new(self.channel.clone())
            .get(proto::GetRootchainPeerAddressesRequest { chain_id: chain_id.to_string() })
            .await
            .map_err(|source| GetRootchainPeerAddressError::Status { source })?
            .into_inner()
            .addresses
            .into_iter()
            .map(PeerAddress::try_from)
            .collect::<Result<HashSet<PeerAddress>, _>>()
            .map_err(GetRootchainPeerAddressError::from)
    }

    async fn insert<S>(
        &self,
        chain_id: S,
        addr: &PeerAddress,
    ) -> Result<(), InsertRootchainPeerAddressError>
    where
        S: fmt::Display + Send + Sync,
    {
        proto::RootchainPeerServiceClient::new(self.channel.clone())
            .insert(proto::InsertRootchainPeerAddressRequest {
                chain_id: chain_id.to_string(),
                address: Some(addr.clone().into()),
            })
            .await
            .map_err(|source| InsertRootchainPeerAddressError::Status { source })?;
        Ok(())
    }

    async fn clear(&self) -> Result<(), ClearRootchainPeerAddressError> {
        proto::RootchainPeerServiceClient::new(self.channel.clone())
            .clear(())
            .await
            .map_err(|source| ClearRootchainPeerAddressError::Status { source })?;

        Ok(())
    }
}
