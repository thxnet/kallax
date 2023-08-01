use std::{collections::HashMap, fmt, sync::Arc};

use kallax_primitives::ExternalEndpoint;
use time::Duration;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PeerAddress {
    address: kallax_primitives::PeerAddress,

    external: Option<ExternalEndpoint>,
}

type PeerAddresses = HashMap<PeerAddress, Option<time::OffsetDateTime>>;

#[derive(Clone, Debug)]
pub struct PeerAddressBook {
    ttl: Duration,

    books: Arc<Mutex<HashMap<String, PeerAddresses>>>,
}

impl Default for PeerAddressBook {
    fn default() -> Self { Self::new() }
}

impl PeerAddressBook {
    pub fn new() -> Self { Self::with_ttl(std::time::Duration::from_secs(120)) }

    pub fn with_ttl(ttl: std::time::Duration) -> Self {
        let ttl = Duration::new(i64::try_from(ttl.as_secs()).unwrap_or_default(), 0);
        Self { ttl, books: Arc::default() }
    }
}

impl PeerAddressBook {
    pub async fn fetch_peers<ChainId>(
        &self,
        chain_id: ChainId,
    ) -> Vec<kallax_primitives::PeerAddress>
    where
        ChainId: fmt::Display,
    {
        let chain_id = chain_id.to_string();
        self.books.lock().await.get(&chain_id).map_or_else(Vec::new, |addresses| {
            addresses.iter().map(|(addr, _)| addr.address.clone()).collect()
        })
    }

    pub async fn fetch_exposed_peers<ChainId>(
        &self,
        chain_id: ChainId,
    ) -> Vec<kallax_primitives::PeerAddress>
    where
        ChainId: fmt::Display,
    {
        let chain_id = chain_id.to_string();
        self.books.lock().await.get(&chain_id).map_or_else(Vec::new, |addresses| {
            addresses
                .iter()
                .filter_map(|(PeerAddress { address, external }, _)| {
                    external
                        .as_ref()
                        .and_then(|external_endpoint| address.exposed(external_endpoint))
                })
                .collect()
        })
    }

    #[allow(dead_code)]
    pub async fn insert_reserved<ChainId>(
        &self,
        chain_id: ChainId,
        peer_address: kallax_primitives::PeerAddress,
        external_endpoint: Option<ExternalEndpoint>,
    ) where
        ChainId: fmt::Display,
    {
        let chain_id = chain_id.to_string();
        self.books
            .lock()
            .await
            .entry(chain_id)
            .or_insert_with(HashMap::new)
            .insert(PeerAddress { address: peer_address, external: external_endpoint }, None);
    }

    pub async fn insert<ChainId>(
        &self,
        chain_id: ChainId,
        peer_address: kallax_primitives::PeerAddress,
        external_endpoint: Option<ExternalEndpoint>,
    ) where
        ChainId: fmt::Display,
    {
        let chain_id = chain_id.to_string();
        self.books.lock().await.entry(chain_id).or_insert_with(HashMap::new).insert(
            PeerAddress { address: peer_address, external: external_endpoint },
            Some(time::OffsetDateTime::now_utc()),
        );
    }

    pub async fn flush(&self) {
        tracing::info!("Start to flush stalled peer addresses");

        let now = time::OffsetDateTime::now_utc();

        let mut books = self.books.lock().await;

        for (_, book) in &mut books.iter_mut() {
            book.retain(|PeerAddress { address, .. }, last_update_time| {
                last_update_time.map_or(true, |last_update_time| {
                    if (now - last_update_time) < self.ttl {
                        true
                    } else {
                        tracing::info!("`{address}` is stalled, removing it");
                        false
                    }
                })
            });
        }
        tracing::info!("Flushing stalled peer addresses completed");
    }

    pub async fn clear(&self) {
        let mut books = self.books.lock().await;
        books.clear();
    }
}
