mod config;
mod error;

use std::time::Duration;

use kallax_network_broker::NodeConfig;
use serde_yaml::{self};

pub use self::{
    config::{Config, ThxnetConfig},
    error::{Error, Result},
};
use crate::network_broker::config::{Mainnet, Testnet};

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
        let thxnet_config: ThxnetConfig =
            serde_yaml::from_reader(config_file).expect("Could not read values.");

        let ThxnetConfig { mainnet, testnet } = thxnet_config;

        let mut nodes_config: Vec<NodeConfig> = vec![];

        mainnet.map(|Mainnet { rootchain, thx, lmt }| {
            rootchain.map(|chain| {
                let mut config = chain.nodes_config(Mainnet::ROOTCHAIN_ID.to_owned());
                nodes_config.append(&mut config);
            });
            thx.map(|chain| {
                let config = &mut chain
                    .nodes_config(Mainnet::ROOTCHAIN_ID.to_owned(), Mainnet::THX_ID.to_owned());
                nodes_config.append(config);
            });
            lmt.map(|chain| {
                let config = &mut chain
                    .nodes_config(Mainnet::ROOTCHAIN_ID.to_owned(), Mainnet::LMT_ID.to_owned());
                nodes_config.append(config);
            });
        });

        testnet.map(|Testnet { rootchain, thx, lmt, txd, sand }| {
            rootchain.map(|chain| {
                let mut config = chain.nodes_config(Testnet::ROOTCHAIN_ID.to_owned());
                nodes_config.append(&mut config);
            });
            thx.map(|chain| {
                let config = &mut chain
                    .nodes_config(Testnet::ROOTCHAIN_ID.to_owned(), Testnet::THX_ID.to_owned());
                nodes_config.append(config);
            });
            lmt.map(|chain| {
                let config = &mut chain
                    .nodes_config(Testnet::ROOTCHAIN_ID.to_owned(), Testnet::LMT_ID.to_owned());
                nodes_config.append(config);
            });
            txd.map(|chain| {
                let config = &mut chain
                    .nodes_config(Testnet::ROOTCHAIN_ID.to_owned(), Testnet::TXD_ID.to_owned());
                nodes_config.append(config);
            });
            sand.map(|chain| {
                let config = &mut chain
                    .nodes_config(Testnet::ROOTCHAIN_ID.to_owned(), Testnet::SAND_ID.to_owned());
                nodes_config.append(config);
            });
        });

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
