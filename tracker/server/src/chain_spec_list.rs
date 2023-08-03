use std::{collections::HashMap, sync::Arc};

use kallax_primitives::{BlockchainLayer, ChainSpec};
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct ChainSpecList {
    blockchain_layer: BlockchainLayer,

    chain_specs: Arc<Mutex<HashMap<String, ChainSpec>>>,
}

impl ChainSpecList {
    pub fn new<C>(blockchain_layer: BlockchainLayer, chain_specs: C) -> Self
    where
        C: IntoIterator<Item = ChainSpec>,
    {
        let chain_specs = Arc::new(Mutex::new(
            chain_specs
                .into_iter()
                .map(|chain_spec| {
                    let id = chain_spec.id().to_string();
                    tracing::info!(
                        "{blockchain_layer} spec `{id}` is loaded, file size: {}",
                        chain_spec.as_ref().len()
                    );
                    (id, chain_spec)
                })
                .collect(),
        ));
        Self { blockchain_layer, chain_specs }
    }

    pub async fn insert(&self, chain_id: &str, spec: ChainSpec) -> bool {
        if self.chain_specs.lock().await.insert(chain_id.to_string(), spec).is_some() {
            tracing::warn!("{} spec `{chain_id}` is replaced by a new one", self.blockchain_layer);
            true
        } else {
            tracing::info!("{} spec `{chain_id}` is added", self.blockchain_layer);
            false
        }
    }

    pub async fn get(&self, chain_id: &str) -> Option<ChainSpec> {
        self.chain_specs.lock().await.get(chain_id).cloned()
    }
}
