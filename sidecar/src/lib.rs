mod error;
mod network_keeper;

use std::{future::Future, time::Duration};

use futures::{future, future::Either, FutureExt, StreamExt};
use kallax_tracker_client::{Client as TrackerClient, Config as TrackerClientConfig};
use snafu::ResultExt;

pub use self::error::{Error, Result};
use self::network_keeper::NetworkKeeper;

#[derive(Clone, Debug)]
pub struct Config {
    tracker_grpc_endpoint: http::Uri,

    polling_interval: Duration,

    rootchain_endpoint: ChainEndpoint,

    leafchain_endpoint: Option<ChainEndpoint>,
}

#[derive(Clone, Debug)]
pub struct ChainEndpoint {
    metadata: kallax_tracker_primitives::chain_spec::ChainMetadata,

    websocket_endpoint: http::Uri,
}

#[derive(Debug)]
pub struct Sidecar {
    polling_interval: Duration,

    rootchain_network_keeper: NetworkKeeper,

    leafchain_network_keeper: Option<NetworkKeeper>,
}

impl Sidecar {
    pub async fn new(config: Config) -> Result<Self> {
        let Config {
            tracker_grpc_endpoint,
            polling_interval,
            rootchain_endpoint,
            leafchain_endpoint,
        } = config;

        let tracker_client = TrackerClient::new(TrackerClientConfig {
            grpc_endpoint: tracker_grpc_endpoint.clone(),
        })
        .await
        .with_context(|_| error::ConnectTrackerSnafu { uri: tracker_grpc_endpoint })?;

        let rootchain_network_keeper = {
            let ChainEndpoint { websocket_endpoint, metadata: chain_metadata } = rootchain_endpoint;
            NetworkKeeper::new(chain_metadata, websocket_endpoint, tracker_client.clone())
        };

        let leafchain_network_keeper = leafchain_endpoint.map(
            |ChainEndpoint { websocket_endpoint, metadata: chain_metadata }| {
                NetworkKeeper::new(chain_metadata, websocket_endpoint, tracker_client)
            },
        );

        Ok(Self { polling_interval, rootchain_network_keeper, leafchain_network_keeper })
    }

    pub async fn serve_with_shutdown<F>(self, shutdown: F) -> Result<()>
    where
        F: Future<Output = ()> + Send + Sync + Unpin,
    {
        let mut shutdown_signal = shutdown.into_stream();
        let Self { polling_interval, rootchain_network_keeper, leafchain_network_keeper } = self;

        loop {
            match future::select(
                shutdown_signal.next().boxed(),
                tokio::time::sleep(polling_interval).boxed(),
            )
            .await
            {
                Either::Left(_) => {
                    tracing::info!("Shutting down");
                    break;
                }
                Either::Right(_) => {
                    if let Err(err) = rootchain_network_keeper.execute().await {
                        tracing::warn!("Error occurs while operating Rootchain node, error: {err}");
                    }

                    if let Some(ref leafchain_network_keeper) = leafchain_network_keeper {
                        if let Err(err) = leafchain_network_keeper.execute().await {
                            tracing::warn!(
                                "Error occurs while operating Leafchain node, error: {err}"
                            );
                        }
                    }
                }
            }
        }

        tracing::info!("Sidecar is down");
        Ok(())
    }
}
