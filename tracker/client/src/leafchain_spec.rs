use std::fmt;

use async_trait::async_trait;
use kallax_primitives::ChainSpec;
use kallax_tracker_proto as proto;

use crate::{error::GetLeafchainSpecError, Client};

#[async_trait]
pub trait LeafchainSpec {
    async fn get<S>(&self, chain_id: S) -> Result<ChainSpec, GetLeafchainSpecError>
    where
        S: fmt::Display + Send + Sync;
}

#[async_trait]
impl LeafchainSpec for Client {
    async fn get<S>(&self, chain_id: S) -> Result<ChainSpec, GetLeafchainSpecError>
    where
        S: fmt::Display + Send + Sync,
    {
        let mut client = proto::LeafchainSpecServiceClient::new(self.channel.clone());

        let resp = client
            .get(proto::GetLeafchainSpecRequest { chain_id: chain_id.to_string() })
            .await
            .map_err(|source| GetLeafchainSpecError::Status { source })?;

        ChainSpec::try_from(resp.into_inner().spec.as_slice()).map_err(GetLeafchainSpecError::from)
    }
}
