mod config;
mod error;

use std::time::Duration;

use futures::FutureExt;
use kallax_sidecar::ChainEndpoint;
use snafu::ResultExt;
use tokio::signal::unix::{signal, SignalKind};

pub use self::{
    config::Config,
    error::{Error, Result},
};

const POLLING_INTERVAL: Duration = Duration::from_millis(1000);

/// # Errors
///
/// This function returns an error if the sidecar is not created.
pub async fn run(config: Config) -> Result<()> {
    let config = {
        let Config {
            tracker_grpc_endpoint,
            rootchain_name,
            rootchain_node_websocket_endpoint,
            leafchain_name,
            leafchain_node_websocket_endpoint,
        } = config;

        let leafchain_endpoint = match (leafchain_name, leafchain_node_websocket_endpoint) {
            (Some(name), Some(websocket_endpoint)) => {
                Some(ChainEndpoint { chain_id: name, websocket_endpoint })
            }
            (Some(_), None) => return Err(Error::LeafchainNodeWebSocketEndpointNotProvided),
            (None, Some(_)) => return Err(Error::LeafchainNameNotProvided),
            (None, None) => None,
        };

        let rootchain_endpoint = ChainEndpoint {
            chain_id: rootchain_name,
            websocket_endpoint: rootchain_node_websocket_endpoint,
        };

        kallax_sidecar::Config {
            tracker_grpc_endpoint,
            polling_interval: POLLING_INTERVAL,
            rootchain_endpoint,
            leafchain_endpoint,
        }
    };
    let sidecar = kallax_sidecar::Sidecar::new(config).await?;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();

    let sidecar_handle = tokio::spawn(async move {
        let shutdown_signal = async move {
            rx.recv().await;
        }
        .boxed();
        sidecar.serve_with_shutdown(shutdown_signal).await;
    });

    tracing::debug!("Create UNIX signal listener for `SIGTERM`");
    let mut sigterm =
        signal(SignalKind::terminate()).context(error::CreateUnixSignalListenerSnafu)?;
    tracing::debug!("Create UNIX signal listener for `SIGINT`");
    let mut sigint =
        signal(SignalKind::interrupt()).context(error::CreateUnixSignalListenerSnafu)?;

    tracing::debug!("Wait for shutdown signal");
    drop(futures::future::select(sigterm.recv().boxed(), sigint.recv().boxed()).await);

    // send shutdown signal to unbounded channel receiver by dropping the sender
    drop(tx);

    sidecar_handle.await.context(error::JoinTaskHandleSnafu)?;

    Ok(())
}
