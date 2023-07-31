mod config;
mod error;

use std::time::Duration;

use kallax_sidecar::ChainEndpoint;

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
            rootchain_id,
            rootchain_node_websocket_endpoint,
            leafchain_id,
            leafchain_node_websocket_endpoint,
            allow_loopback_ip,
            exposed_rootchain_p2p_port,
            exposed_leafchain_p2p_port,
        } = config;

        let leafchain_endpoint = match (leafchain_id, leafchain_node_websocket_endpoint) {
            (Some(chain_id), Some(websocket_endpoint)) => {
                Some(ChainEndpoint { chain_id, websocket_endpoint })
            }
            (Some(_), None) => return Err(Error::LeafchainNodeWebSocketEndpointNotProvided),
            (None, Some(_)) => return Err(Error::LeafchainNameNotProvided),
            (None, None) => None,
        };

        let rootchain_endpoint = ChainEndpoint {
            chain_id: rootchain_id,
            websocket_endpoint: rootchain_node_websocket_endpoint,
        };

        kallax_sidecar::Config {
            tracker_grpc_endpoint,
            polling_interval: POLLING_INTERVAL,
            rootchain_endpoint,
            leafchain_endpoint,
            allow_loopback_ip,
            exposed_rootchain_p2p_port,
            exposed_leafchain_p2p_port,
        }
    };

    kallax_sidecar::serve(config).await?;

    Ok(())
}
