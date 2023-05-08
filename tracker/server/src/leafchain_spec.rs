use std::{collections::HashMap, sync::Arc};

use kallax_primitives::ChainSpec;
use kallax_tracker_proto as proto;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct Service {
    chain_specs: Arc<Mutex<HashMap<String, ChainSpec>>>,
}

impl Service {
    pub fn new<C>(chain_specs: C) -> Self
    where
        C: IntoIterator<Item = ChainSpec>,
    {
        let chain_specs = Arc::new(Mutex::new(
            chain_specs
                .into_iter()
                .map(|chain_spec| {
                    let id = chain_spec.id().to_string();
                    tracing::info!(
                        "Leafchain spec `{id}` is loaded, file size: {}",
                        chain_spec.as_ref().len()
                    );
                    (id, chain_spec)
                })
                .collect(),
        ));
        Self { chain_specs }
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

        if self.chain_specs.lock().await.insert(chain_id.clone(), spec).is_some() {
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

        if let Some(spec) = self.chain_specs.lock().await.get(&chain_id) {
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
