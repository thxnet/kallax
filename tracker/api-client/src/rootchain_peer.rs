use std::{collections::HashSet, fmt};

use async_trait::async_trait;
use kallax_primitives::PeerAddress;
use url::Url;

use crate::{error::GetRootchainPeerAddressError, Client};

#[async_trait]
pub trait RootchainPeer {
    async fn get<S>(
        &self,
        chain_id: S,
    ) -> Result<HashSet<PeerAddress>, GetRootchainPeerAddressError>
    where
        S: fmt::Display + Send + Sync;
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
        let Self { client: api_client, api_endpoint } = self;

        let mut url = Url::parse(api_endpoint.to_string().as_str())
            .map_err(|source| GetRootchainPeerAddressError::UrlParse { source })?;

        url.path_segments_mut()
            .map_err(|_| GetRootchainPeerAddressError::UrlCanNotBeBase)?
            .pop_if_empty()
            .push("rootchain")
            .push(chain_id.to_string().as_str())
            .push("peers");

        api_client
            .get(url)
            .send()
            .await
            .map_err(|source| GetRootchainPeerAddressError::Reqwest { source })?
            .json::<Vec<PeerAddress>>()
            .await
            .map_err(|source| GetRootchainPeerAddressError::Reqwest { source })
            .map(|vec| vec.into_iter().collect::<HashSet<PeerAddress>>())
    }
}
