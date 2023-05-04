use std::collections::HashMap;

use kallax_tracker_primitives::chain_spec::{ChainMetadata, ChainSpec};
use kallax_tracker_proto::chain_spec as proto;
use tonic::{Request, Response, Status};

use crate::error;

#[derive(Default)]
pub struct Service {
    chain_specs: HashMap<ChainMetadata, ChainSpec>,
}

#[tonic::async_trait]
impl proto::ChainSpecService for Service {
    async fn get(
        &self,
        req: Request<proto::GetChainSpecRequest>,
    ) -> Result<Response<proto::GetChainSpecResponse>, Status> {
        let metadata = {
            let metadata = req
                .into_inner()
                .metadata
                .ok_or_else(|| Status::invalid_argument("chain metadata not found"))?;

            ChainMetadata::try_from(metadata).map_err(error::into_invalid_argument_status)?
        };

        match self.chain_specs.get(&metadata) {
            Some(spec) => Ok(Response::new(proto::GetChainSpecResponse {
                metadata: Some(proto::ChainMetadata::from(metadata)),
                spec: Some(proto::ChainSpec::from(spec.clone())),
            })),
            None => {
                let message = format!("chain spec `{}` not found", metadata.name);
                Err(Status::not_found(message))
            }
        }
    }
}
