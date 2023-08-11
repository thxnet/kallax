mod config;
mod error;
mod lib;

use std::time::Duration;

use kallax_network_broker::NodeConfig;
use serde_yaml::{self};

pub use self::{
    config::Config,
    error::{Error, Result},
    lib::Thxnet,
};

const POLLING_INTERVAL: Duration = Duration::from_millis(1000);
const TRACKER_API_ENDPOINT: &str = "https://tracker.testnet.thxnet.org/api/v1/";
const CONFIG_PATH: &str = "./network-broker.yaml";

/// # Errors
///
/// This function returns an error if the network-broker is not created.
pub async fn run(config: Config) -> Result<()> {
    let config = {
        let Config { tracker_grpc_endpoint, file } = config;

        tracing::info!("Read config: {file:?}");

        let config_file = std::fs::File::open(&file).expect("Could not open file.");
        let thxnet: Thxnet = serde_yaml::from_reader(config_file).expect("Could not read values.");

        let nodes_config: Vec<NodeConfig> = thxnet.nodes_config();

        tracing::info!("{nodes_config:?}");

        kallax_network_broker::Config {
            tracker_grpc_endpoint,
            polling_interval: POLLING_INTERVAL,
            nodes: nodes_config,
        }
    };

    kallax_network_broker::serve(config).await?;

    Ok(())
}
