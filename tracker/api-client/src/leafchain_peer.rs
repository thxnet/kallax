use std::{collections::HashSet, fmt};

use async_trait::async_trait;
use kallax_primitives::{ExternalEndpoint, PeerAddress};
use kallax_tracker_server::InsertLeafchainPeerAddressRequest;
use reqwest::Url;

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

        let mut url =
            Url::parse(api_endpoint.to_string().as_str()).expect("parse url error: {api_endpoint}");

        url.set_path(format!("/api/v1/leafchain/{chain_id}/peers").as_str());

        let peers = api_client
            .get(url)
            .send()
            .await
            .expect("get response error")
            .json::<Vec<PeerAddress>>()
            .await
            .expect("parse json error")
            .into_iter()
            .collect::<HashSet<PeerAddress>>();

        Ok(peers)
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

        let mut url =
            Url::parse(api_endpoint.to_string().as_str()).expect("parse url error: {api_endpoint}");

        url.set_path(format!("/api/v1/leafchain/{chain_id}/insert").as_str());

        api_client
            .post(url)
            .json(&InsertLeafchainPeerAddressRequest {
                peer_address: addr.clone(),
                external_endpoint: external_endpoint.clone().unwrap(),
            })
            .send()
            .await
            .expect("get response error");

        Ok(())
    }
}
