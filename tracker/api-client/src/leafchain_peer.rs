use std::{collections::HashSet, fmt};

use async_trait::async_trait;
use kallax_primitives::PeerAddress;
use url::Url;

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

        let mut url = Url::parse(api_endpoint.to_string().as_str())
            .map_err(|source| GetLeafchainPeerAddressError::UrlParse { source })?;

        url.path_segments_mut()
            .map_err(|source| GetLeafchainPeerAddressError::UrlCanNotBeBase { source })?
            .pop_if_empty()
            .push("leafchain")
            .push(chain_id.to_string().as_str())
            .push("peers");

        api_client
            .get(url)
            .send()
            .await
            .map_err(|source| GetLeafchainPeerAddressError::Reqwest { source })?
            .json::<Vec<PeerAddress>>()
            .await
            .map_err(|source| GetLeafchainPeerAddressError::Reqwest { source })
            .map(|vec| vec.into_iter().collect::<HashSet<PeerAddress>>())
    }
}
