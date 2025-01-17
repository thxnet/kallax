use kallax_network_broker::{ChainEndpoint, Node};
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
    pub activa: Option<Leafchain>,
    pub mirrored_body: Option<Leafchain>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Testnet {
    pub rootchain: Option<Rootchain>,
    pub thx: Option<Leafchain>,
    pub lmt: Option<Leafchain>,
    pub txd: Option<Leafchain>,
    pub sand: Option<Leafchain>,
    pub aether: Option<Leafchain>,
    pub izutsuya: Option<Leafchain>,
    pub mirrored_body: Option<Leafchain>,
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

    pub external_p2p_endpoint: Option<P2pEndpoint>,
}

#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeafchainNode {
    pub rootchain_ws_endpoint: String,
    pub leafchain_ws_endpoint: String,

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
    pub fn nodes(&self) -> Vec<Node> {
        let mut nodes: Vec<Node> = Vec::new();
        let Self { mainnet, testnet } = self;

        if let Some(net) = mainnet {
            let mut config = net.nodes();
            nodes.append(&mut config);
        }

        if let Some(net) = testnet {
            let mut config = net.nodes();
            nodes.append(&mut config);
        }

        nodes
    }
}

impl Mainnet {
    pub const ACTIVA_ID: &'static str = "activa_mainnet";
    pub const LMT_ID: &'static str = "lmt_mainnet";
    pub const MIRRORED_BODY_ID: &'static str = "mirrored_body_mainnet";
    pub const ROOTCHAIN_ID: &'static str = "thxnet_mainnet";
    pub const THX_ID: &'static str = "thx_mainnet";

    pub fn nodes(&self) -> Vec<Node> {
        let mut nodes: Vec<Node> = Vec::new();
        let Self { rootchain, thx, lmt, activa, mirrored_body } = self;

        if let Some(chain) = rootchain {
            let mut config = chain.nodes(Self::ROOTCHAIN_ID);
            nodes.append(&mut config);
        }

        if let Some(chain) = thx {
            let config = &mut chain.nodes(Self::ROOTCHAIN_ID, Self::THX_ID);
            nodes.append(config);
        }

        if let Some(chain) = lmt {
            let config = &mut chain.nodes(Self::ROOTCHAIN_ID, Self::LMT_ID);
            nodes.append(config);
        }

        if let Some(chain) = activa {
            let config = &mut chain.nodes(Self::ROOTCHAIN_ID, Self::ACTIVA_ID);
            nodes.append(config);
        }

        if let Some(chain) = mirrored_body {
            let config = &mut chain.nodes(Self::ROOTCHAIN_ID, Self::MIRRORED_BODY_ID);
            nodes.append(config);
        }

        nodes
    }
}

impl Testnet {
    pub const AETHER_ID: &'static str = "aether_testnet";
    pub const IZUTSUYA_ID: &'static str = "izutsuya_testnet";
    pub const LMT_ID: &'static str = "lmt_testnet";
    pub const MIRRORED_BODY_ID: &'static str = "mirrored_body_testnet";
    pub const ROOTCHAIN_ID: &'static str = "thxnet_testnet";
    pub const SAND_ID: &'static str = "sand_testnet";
    pub const THX_ID: &'static str = "thx_testnet";
    pub const TXD_ID: &'static str = "txd_testnet";

    pub fn nodes(&self) -> Vec<Node> {
        let mut nodes: Vec<Node> = Vec::new();
        let Self { rootchain, thx, lmt, txd, sand, aether, izutsuya, mirrored_body } = self;

        if let Some(chain) = rootchain {
            let mut config = chain.nodes(Self::ROOTCHAIN_ID);
            nodes.append(&mut config);
        }

        if let Some(chain) = thx {
            let config = &mut chain.nodes(Self::ROOTCHAIN_ID, Self::THX_ID);
            nodes.append(config);
        }

        if let Some(chain) = lmt {
            let config = &mut chain.nodes(Self::ROOTCHAIN_ID, Self::LMT_ID);
            nodes.append(config);
        }

        if let Some(chain) = txd {
            let config = &mut chain.nodes(Self::ROOTCHAIN_ID, Self::TXD_ID);
            nodes.append(config);
        }

        if let Some(chain) = sand {
            let config = &mut chain.nodes(Self::ROOTCHAIN_ID, Self::SAND_ID);
            nodes.append(config);
        }

        if let Some(chain) = aether {
            let config = &mut chain.nodes(Self::ROOTCHAIN_ID, Self::AETHER_ID);
            nodes.append(config);
        }

        if let Some(chain) = izutsuya {
            let config = &mut chain.nodes(Self::ROOTCHAIN_ID, Self::IZUTSUYA_ID);
            nodes.append(config);
        }

        if let Some(chain) = mirrored_body {
            let config = &mut chain.nodes(Self::ROOTCHAIN_ID, Self::MIRRORED_BODY_ID);
            nodes.append(config);
        }

        nodes
    }
}

impl Rootchain {
    pub fn nodes(&self, rootchain_id: &str) -> Vec<Node> {
        if self.validators.is_none() && self.archives.is_none() {
            return Vec::new();
        }

        let mut rootchain_nodes: Vec<RootchainNode> = Vec::new();

        let Self { validators, archives } = self;

        if let Some(nodes) = validators {
            rootchain_nodes.append(&mut nodes.clone());
        };

        if let Some(nodes) = archives {
            rootchain_nodes.append(&mut nodes.clone());
        };

        rootchain_nodes
            .into_iter()
            .map(|RootchainNode { ws_endpoint, external_p2p_endpoint }| {
                let external_rootchain_p2p_endpoint =
                    if let Some(P2pEndpoint { host, port }) = external_p2p_endpoint {
                        Some(ExternalEndpoint { host, port })
                    } else {
                        None
                    };
                Node {
                    rootchain_endpoint: ChainEndpoint {
                        chain_id: rootchain_id.to_string(),
                        websocket_endpoint: ws_endpoint
                            .parse::<http::Uri>()
                            .expect("websocket endpoint is invalid"),
                    },
                    leafchain_endpoint: None,
                    external_rootchain_p2p_endpoint,
                    external_leafchain_p2p_endpoint: None,
                }
            })
            .collect()
    }
}

impl Leafchain {
    pub fn nodes(&self, rootchain_id: &str, leafchain_id: &str) -> Vec<Node> {
        if self.collators.is_none() && self.archives.is_none() {
            return Vec::new();
        }

        let mut leafchain_nodes: Vec<LeafchainNode> = Vec::new();

        let Self { collators, archives } = self;

        if let Some(nodes) = collators {
            leafchain_nodes.append(&mut nodes.clone());
        };

        if let Some(nodes) = archives {
            leafchain_nodes.append(&mut nodes.clone());
        };

        leafchain_nodes
            .into_iter()
            .map(
                |LeafchainNode {
                     rootchain_ws_endpoint,
                     leafchain_ws_endpoint,
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

                    Node {
                        rootchain_endpoint: ChainEndpoint {
                            chain_id: rootchain_id.to_string(),
                            websocket_endpoint: rootchain_ws_endpoint
                                .parse::<http::Uri>()
                                .expect("websocket endpoint is invalid"),
                        },
                        leafchain_endpoint: Some(ChainEndpoint {
                            chain_id: leafchain_id.to_string(),
                            websocket_endpoint: leafchain_ws_endpoint
                                .parse::<http::Uri>()
                                .expect("websocket endpoint is invalid"),
                        }),
                        external_rootchain_p2p_endpoint,
                        external_leafchain_p2p_endpoint,
                    }
                },
            )
            .collect()
    }
}
