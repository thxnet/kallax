mod error;

use async_trait::async_trait;
use kallax_tracker_primitives::chain_spec::{ChainMetadata, ChainSpec};
use kallax_tracker_proto::chain_spec as proto;
use snafu::{OptionExt, ResultExt};

pub use self::error::Error;
use self::error::Result;
use crate::Client;

#[async_trait]
pub trait ChainSpecExt {
    async fn get(&self, chain_metadata: ChainMetadata) -> Result<ChainSpec>;
}

#[async_trait]
impl ChainSpecExt for Client {
    async fn get(&self, chain_metadata: ChainMetadata) -> Result<ChainSpec> {
        let mut client = proto::ChainSpecServiceClient::new(self.channel.clone());

        let spec = client
            .get(proto::GetChainSpecRequest {
                metadata: Some(proto::ChainMetadata::from(chain_metadata)),
            })
            .await
            .with_context(|_| error::GetChainSpecSnafu)?
            .into_inner()
            .spec
            .context(error::MissingChainSpecSnafu)?;

        ChainSpec::try_from(spec).map_err(Error::from)
    }
}
