mod config;
mod error;
mod options;

use std::{path::PathBuf, time::Duration};

use kallax_network_broker::Node;
use serde_yaml::{self};
use tokio::fs;

pub use self::{
    config::Thxnet,
    error::{Error, Result},
    options::Options,
};

const POLLING_INTERVAL: Duration = Duration::from_millis(5000);
pub const TRACKER_API_ENDPOINT: &str = "https://tracker.testnet.thxnet.org";
pub const CONFIG_PATH: &str = "./network-broker.yaml";

/// # Errors
///
/// This function returns an error if the network-broker is not created.
pub async fn run(tracker_api_endpoint: http::Uri, file: PathBuf) -> Result<()> {
    let config = {
        tracing::info!("Read configuration file from `{}`", file.display());

        let config_file = fs::read(&file).await.expect("read config error");
        let thxnet: Thxnet = serde_yaml::from_slice(config_file.as_slice())
            .map_err(|source| Error::SerdeYaml { source, path: file.display().to_string() })?;

        let nodes: Vec<Node> = thxnet.nodes();

        tracing::info!("{nodes:?}");

        kallax_network_broker::Config {
            tracker_api_endpoint,
            polling_interval: POLLING_INTERVAL,
            nodes,
        }
    };

    kallax_network_broker::serve(config).await?;

    Ok(())
}
