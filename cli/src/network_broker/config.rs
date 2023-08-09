use clap::Args;
use kallax_network_broker::{ChainEndpoint, NodeConfig};
use kallax_primitives::ExternalEndpoint;
use serde::{Deserialize, Serialize};

use crate::network_broker::{CONFIG_PATH, TRACKER_API_ENDPOINT};

#[derive(Args, Debug)]
pub struct Config {
    #[clap(long = "tracker-grpc-endpoint", help = "Tracker gRPC endpoint", default_value_t = TRACKER_API_ENDPOINT.parse::<http::Uri>().unwrap())]
    pub tracker_grpc_endpoint: http::Uri,

    #[clap(long = "file", help = "Config file path", default_value_t = String::from(CONFIG_PATH))]
    pub file: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThxnetConfig {
    pub mainnet: Option<Mainnet>,
    pub testnet: Option<Testnet>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mainnet {
    pub rootchain: Option<Rootchain>,
    pub thx: Option<Leafchain>,
    pub lmt: Option<Leafchain>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Testnet {
    pub rootchain: Option<Rootchain>,
    pub thx: Option<Leafchain>,
    pub lmt: Option<Leafchain>,
    pub txd: Option<Leafchain>,
    pub sand: Option<Leafchain>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rootchain {
    pub validators: Option<Vec<RootchainNode>>,
    pub archives: Option<Vec<RootchainNode>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Leafchain {
    pub collators: Option<Vec<LeafchainNode>>,
    pub archives: Option<Vec<LeafchainNode>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootchainNode {
    pub ws_endpoint: String,

    pub allow_loopback_ip: Option<bool>,

    pub external_p2p_host: Option<String>,
    pub external_p2p_port: Option<u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeafchainNode {
    pub rootchain_ws_endpoint: String,
    pub leafchain_ws_endpoint: String,

    pub allow_loopback_ip: Option<bool>,

    pub external_rootchain_p2p_host: Option<String>,
    pub external_rootchain_p2p_port: Option<u16>,

    pub external_leafchain_p2p_host: Option<String>,
    pub external_leafchain_p2p_port: Option<u16>,
}

impl Mainnet {
    pub const LMT_ID: &str = "lmt_mainnet";
    pub const ROOTCHAIN_ID: &str = "thxnet_mainnet";
    pub const THX_ID: &str = "thx_mainnet";
}

impl Testnet {
    pub const LMT_ID: &str = "lmt_testnet";
    pub const ROOTCHAIN_ID: &str = "thxnet_testnet";
    pub const SAND_ID: &str = "lmt_testnet";
    pub const THX_ID: &str = "thx_testnet";
    pub const TXD_ID: &str = "thx_testnet";
}

impl Rootchain {
    pub fn nodes_config(&self, rootchain_id: String) -> Vec<NodeConfig> {
        if self.validators.is_none() && self.archives.is_none() {
            return vec![];
        }

        let mut rootchain_nodes: Vec<RootchainNode> = vec![];

        let Rootchain { validators, archives } = self;

        if validators.is_some() {
            let nodes = validators.clone();
            rootchain_nodes.append(&mut nodes.unwrap());
        }

        if archives.is_some() {
            let nodes = archives.clone();
            rootchain_nodes.append(&mut nodes.unwrap());
        }

        rootchain_nodes
            .into_iter()
            .map(
                |RootchainNode {
                     ws_endpoint,
                     mut allow_loopback_ip,
                     external_p2p_host,
                     external_p2p_port,
                 }| {
                    let external_rootchain_p2p_endpoint =
                        if external_p2p_host.is_some() && external_p2p_port.is_some() {
                            Some(ExternalEndpoint {
                                host: external_p2p_host.unwrap(),
                                port: external_p2p_port.unwrap(),
                            })
                        } else {
                            None
                        };
                    NodeConfig {
                        rootchain_endpoint: ChainEndpoint {
                            chain_id: rootchain_id.to_owned(),
                            websocket_endpoint: ws_endpoint.parse::<http::Uri>().unwrap(),
                        },
                        leafchain_endpoint: None,
                        allow_loopback_ip: allow_loopback_ip.get_or_insert(false).to_owned(),
                        external_rootchain_p2p_endpoint,
                        external_leafchain_p2p_endpoint: None,
                    }
                },
            )
            .collect()
    }
}

impl Leafchain {
    pub fn nodes_config(&self, rootchain_id: String, leafchain_id: String) -> Vec<NodeConfig> {
        if self.collators.is_none() && self.archives.is_none() {
            return vec![];
        }

        let mut leafchain_nodes: Vec<LeafchainNode> = vec![];

        let Leafchain { collators, archives } = self;

        if collators.is_some() {
            let nodes: Option<Vec<LeafchainNode>> = collators.clone();
            leafchain_nodes.append(&mut nodes.unwrap());
        }

        if archives.is_some() {
            let nodes = archives.clone();
            leafchain_nodes.append(&mut nodes.unwrap());
        }

        leafchain_nodes
            .into_iter()
            .map(
                |LeafchainNode {
                     rootchain_ws_endpoint,
                     leafchain_ws_endpoint,
                     mut allow_loopback_ip,
                     external_rootchain_p2p_host,
                     external_rootchain_p2p_port,
                     external_leafchain_p2p_host,
                     external_leafchain_p2p_port,
                 }| {
                    let external_rootchain_p2p_endpoint = if external_rootchain_p2p_host.is_some()
                        && external_rootchain_p2p_port.is_some()
                    {
                        Some(ExternalEndpoint {
                            host: external_rootchain_p2p_host.unwrap(),
                            port: external_rootchain_p2p_port.unwrap(),
                        })
                    } else {
                        None
                    };

                    let external_leafchain_p2p_endpoint = if external_leafchain_p2p_host.is_some()
                        && external_leafchain_p2p_port.is_some()
                    {
                        Some(ExternalEndpoint {
                            host: external_leafchain_p2p_host.unwrap(),
                            port: external_leafchain_p2p_port.unwrap(),
                        })
                    } else {
                        None
                    };

                    NodeConfig {
                        rootchain_endpoint: ChainEndpoint {
                            chain_id: rootchain_id.to_owned(),
                            websocket_endpoint: rootchain_ws_endpoint.parse::<http::Uri>().unwrap(),
                        },
                        leafchain_endpoint: Some(ChainEndpoint {
                            chain_id: leafchain_id.to_owned(),
                            websocket_endpoint: leafchain_ws_endpoint.parse::<http::Uri>().unwrap(),
                        }),
                        allow_loopback_ip: allow_loopback_ip.get_or_insert(false).to_owned(),
                        external_rootchain_p2p_endpoint,
                        external_leafchain_p2p_endpoint,
                    }
                },
            )
            .collect()
    }
}
