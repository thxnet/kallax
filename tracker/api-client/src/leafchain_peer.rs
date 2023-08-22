use std::{collections::HashSet, fmt};

use async_trait::async_trait;
use kallax_primitives::{ExternalEndpoint, PeerAddress};
use kallax_tracker_server::InsertLeafchainPeerAddressRequest;
use url::Url;

use crate::{
    error::{GetLeafchainPeerAddressError, InsertLeafchainPeerAddressError},
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
        chain_id: S,
        addr: &PeerAddress,
        external_endpoint: &Option<ExternalEndpoint>,
    ) -> Result<(), InsertLeafchainPeerAddressError>
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
            .map_err(|_| GetLeafchainPeerAddressError::UrlCanNotBeBase)?
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

    async fn insert<S>(
        &self,
        chain_id: S,
        addr: &PeerAddress,
        external_endpoint: &Option<ExternalEndpoint>,
    ) -> Result<(), InsertLeafchainPeerAddressError>
    where
        S: fmt::Display + Send + Sync,
    {
        let Self { client: api_client, api_endpoint } = self;

        let mut url = Url::parse(api_endpoint.to_string().as_str())
            .map_err(|source| InsertLeafchainPeerAddressError::UrlParse { source })?;

        url.path_segments_mut()
            .map_err(|_| InsertLeafchainPeerAddressError::UrlCanNotBeBase)?
            .pop_if_empty()
            .push("leafchain")
            .push(chain_id.to_string().as_str())
            .push("insert");

        api_client
            .post(url)
            .json(&InsertLeafchainPeerAddressRequest {
                peer_address: addr.clone(),
                external_endpoint: external_endpoint.clone().unwrap(),
            })
            .send()
            .await
            .map_err(|source| InsertLeafchainPeerAddressError::Reqwest { source })?;

        Ok(())
    }
}
