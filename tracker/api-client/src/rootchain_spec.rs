use std::fmt;

use async_trait::async_trait;
use kallax_primitives::ChainSpec;
use kallax_tracker_proto as proto;

use crate::{error::GetRootchainSpecError, Client};

#[async_trait]
pub trait RootchainSpec {
    async fn get<S>(&self, chain_name: S) -> Result<ChainSpec, GetRootchainSpecError>
    where
        S: fmt::Display + Send + Sync;
}

#[async_trait]
impl RootchainSpec for Client {
    async fn get<S>(&self, chain_id: S) -> Result<ChainSpec, GetRootchainSpecError>
    where
        S: fmt::Display + Send + Sync,
    {
        let resp = proto::RootchainSpecServiceClient::new(self.channel.clone())
            .get(proto::GetRootchainSpecRequest { chain_id: chain_id.to_string() })
            .await
            .map_err(|source| GetRootchainSpecError::Status { source })?;

        ChainSpec::try_from(resp.into_inner().spec.as_slice()).map_err(GetRootchainSpecError::from)
    }
}
