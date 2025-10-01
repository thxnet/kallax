use kallax_primitives::ChainSpec;
use kallax_tracker_proto as proto;
use tonic::{Request, Response, Status};

use crate::chain_spec_list::ChainSpecList;

pub struct Service {
    chain_spec_list: ChainSpecList,
}

impl Service {
    pub const fn new(chain_spec_list: ChainSpecList) -> Self {
        Self { chain_spec_list }
    }
}

#[tonic::async_trait]
impl proto::LeafchainSpecService for Service {
    async fn insert(
        &self,
        req: Request<proto::InsertLeafchainSpecRequest>,
    ) -> Result<Response<proto::InsertLeafchainSpecResponse>, Status> {
        let proto::InsertLeafchainSpecRequest { chain_id, spec } = req.into_inner();

        let spec = ChainSpec::try_from(spec.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        if self.chain_spec_list.insert(&chain_id, spec).await {
            tracing::warn!("Leafchain spec `{chain_id}` is replaced by a new one");
        } else {
            tracing::info!("Leafchain spec `{chain_id}` is added");
        }

        Ok(Response::new(proto::InsertLeafchainSpecResponse { chain_id }))
    }

    async fn get(
        &self,
        req: Request<proto::GetLeafchainSpecRequest>,
    ) -> Result<Response<proto::GetLeafchainSpecResponse>, Status> {
        let chain_id = req.into_inner().chain_id;

        if let Some(spec) = self.chain_spec_list.get(&chain_id).await {
            Ok(Response::new(proto::GetLeafchainSpecResponse {
                chain_id,
                spec: spec.as_ref().to_vec(),
            }))
        } else {
            let message = format!("chain spec `{chain_id}` not found");
            Err(Status::not_found(message))
        }
    }
}
