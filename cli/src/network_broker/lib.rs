use kallax_network_broker::{ChainEndpoint, NodeConfig};
use kallax_primitives::ExternalEndpoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thxnet {
    pub mainnet: Option<Mainnet>,
    pub testnet: Option<Testnet>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mainnet {
    pub rootchain: Option<Rootchain>,
    pub thx: Option<Leafchain>,
    pub lmt: Option<Leafchain>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Testnet {
    pub rootchain: Option<Rootchain>,
    pub thx: Option<Leafchain>,
    pub lmt: Option<Leafchain>,
    pub txd: Option<Leafchain>,
    pub sand: Option<Leafchain>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rootchain {
    pub validators: Option<Vec<RootchainNode>>,
    pub archives: Option<Vec<RootchainNode>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

    pub external_p2p_endpoint: Option<P2pEndpoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeafchainNode {
    pub rootchain_ws_endpoint: String,
    pub leafchain_ws_endpoint: String,

    pub allow_loopback_ip: Option<bool>,

    pub external_rootchain_p2p_endpoint: Option<P2pEndpoint>,
    pub external_leafchain_p2p_endpoint: Option<P2pEndpoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct P2pEndpoint {
    pub host: String,
    pub port: u16,
}

impl Thxnet {
    pub fn nodes_config(&self) -> Vec<NodeConfig> {
        let mut nodes_config: Vec<NodeConfig> = vec![];
        let Self { mainnet, testnet } = self;

        if let Some(net) = mainnet.clone() {
            let mut config = net.nodes_config();
            nodes_config.append(&mut config);
        }

        if let Some(net) = testnet.clone() {
            let mut config = net.nodes_config();
            nodes_config.append(&mut config);
        }

        nodes_config
    }
}

impl Mainnet {
    pub const LMT_ID: &str = "lmt_mainnet";
    pub const ROOTCHAIN_ID: &str = "thxnet_mainnet";
    pub const THX_ID: &str = "thx_mainnet";

    pub fn nodes_config(&self) -> Vec<NodeConfig> {
        let mut nodes_config: Vec<NodeConfig> = vec![];
        let Self { rootchain, thx, lmt } = self;

        if let Some(chain) = rootchain.clone() {
            let mut config = chain.nodes_config(Self::ROOTCHAIN_ID);
            nodes_config.append(&mut config);
        }

        if let Some(chain) = thx.clone() {
            let config = &mut chain.nodes_config(Self::ROOTCHAIN_ID, Self::THX_ID);
            nodes_config.append(config);
        }

        if let Some(chain) = lmt.clone() {
            let config = &mut chain.nodes_config(Self::ROOTCHAIN_ID, Self::LMT_ID);
            nodes_config.append(config);
        }

        nodes_config
    }
}

impl Testnet {
    pub const LMT_ID: &str = "lmt_testnet";
    pub const ROOTCHAIN_ID: &str = "thxnet_testnet";
    pub const SAND_ID: &str = "sand_testnet";
    pub const THX_ID: &str = "thx_testnet";
    pub const TXD_ID: &str = "txd_testnet";

    pub fn nodes_config(&self) -> Vec<NodeConfig> {
        let mut nodes_config: Vec<NodeConfig> = vec![];
        let Self { rootchain, thx, lmt, txd, sand } = self;

        if let Some(chain) = rootchain.clone() {
            let mut config = chain.nodes_config(Self::ROOTCHAIN_ID);
            nodes_config.append(&mut config);
        }

        if let Some(chain) = thx.clone() {
            let config = &mut chain.nodes_config(Self::ROOTCHAIN_ID, Self::THX_ID);
            nodes_config.append(config);
        }

        if let Some(chain) = lmt.clone() {
            let config = &mut chain.nodes_config(Self::ROOTCHAIN_ID, Self::LMT_ID);
            nodes_config.append(config);
        }

        if let Some(chain) = txd.clone() {
            let config = &mut chain.nodes_config(Self::ROOTCHAIN_ID, Self::TXD_ID);
            nodes_config.append(config);
        }

        if let Some(chain) = sand.clone() {
            let config = &mut chain.nodes_config(Self::ROOTCHAIN_ID, Self::SAND_ID);
            nodes_config.append(config);
        }
        nodes_config
    }
}

impl Rootchain {
    pub fn nodes_config(&self, rootchain_id: &str) -> Vec<NodeConfig> {
        if self.validators.is_none() && self.archives.is_none() {
            return vec![];
        }

        let mut rootchain_nodes: Vec<RootchainNode> = vec![];

        let Self { validators, archives } = self;

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
            .map(|RootchainNode { ws_endpoint, mut allow_loopback_ip, external_p2p_endpoint }| {
                let external_rootchain_p2p_endpoint =
                    if let Some(P2pEndpoint { host, port }) = external_p2p_endpoint {
                        Some(ExternalEndpoint { host, port })
                    } else {
                        None
                    };
                NodeConfig {
                    rootchain_endpoint: ChainEndpoint {
                        chain_id: rootchain_id.to_string(),
                        websocket_endpoint: ws_endpoint.parse::<http::Uri>().unwrap(),
                    },
                    leafchain_endpoint: None,
                    allow_loopback_ip: allow_loopback_ip.get_or_insert(false).to_owned(),
                    external_rootchain_p2p_endpoint,
                    external_leafchain_p2p_endpoint: None,
                }
            })
            .collect()
    }
}

impl Leafchain {
    pub fn nodes_config(&self, rootchain_id: &str, leafchain_id: &str) -> Vec<NodeConfig> {
        if self.collators.is_none() && self.archives.is_none() {
            return vec![];
        }

        let mut leafchain_nodes: Vec<LeafchainNode> = vec![];

        let Self { collators, archives } = self;

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
                     external_rootchain_p2p_endpoint,
                     external_leafchain_p2p_endpoint,
                 }| {
                    let external_rootchain_p2p_endpoint =
                        if let Some(P2pEndpoint { host, port }) = external_rootchain_p2p_endpoint {
                            Some(ExternalEndpoint { host, port })
                        } else {
                            None
                        };

                    let external_leafchain_p2p_endpoint =
                        if let Some(P2pEndpoint { host, port }) = external_leafchain_p2p_endpoint {
                            Some(ExternalEndpoint { host, port })
                        } else {
                            None
                        };

                    NodeConfig {
                        rootchain_endpoint: ChainEndpoint {
                            chain_id: rootchain_id.to_string(),
                            websocket_endpoint: rootchain_ws_endpoint.parse::<http::Uri>().unwrap(),
                        },
                        leafchain_endpoint: Some(ChainEndpoint {
                            chain_id: leafchain_id.to_string(),
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
