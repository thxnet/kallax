use std::{collections::HashSet, fmt};

use async_trait::async_trait;
use kallax_primitives::PeerAddress;

use crate::{error::GetLeafchainPeerAddressError, Client};

#[async_trait]
pub trait LeafchainPeer {
    async fn get<S>(
        &self,
        chain_name: S,
    ) -> Result<HashSet<PeerAddress>, GetLeafchainPeerAddressError>
    where
        S: fmt::Display + Send + Sync;
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
        let Self { client: api_client, api_endpoint } = self;

        let endpoint = format!("{api_endpoint}/leafchain/{chain_id}/peers");

        api_client
            .get(endpoint)
            .send()
            .await
            .map_err(|source| GetLeafchainPeerAddressError::Error { source })?
            .json::<Vec<PeerAddress>>()
            .await
            .map_err(|source| GetLeafchainPeerAddressError::Error { source })
            .map(|vec| vec.into_iter().collect::<HashSet<PeerAddress>>())
    }
}
