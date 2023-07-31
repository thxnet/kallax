use clap::Args;

#[derive(Args, Debug)]
pub struct Config {
    #[clap(long = "tracker-grpc-endpoint", help = "Tracker gRPC endpoint")]
    pub tracker_grpc_endpoint: http::Uri,

    #[clap(long = "rootchain-id", help = "Rootchain ID")]
    pub rootchain_id: String,

    #[clap(long = "rootchain-node-websocket-endpoint", help = "Rootchain node WebSocket endpoint")]
    pub rootchain_node_websocket_endpoint: http::Uri,

    #[clap(long = "leafchain-id", help = "Leafchain ID")]
    pub leafchain_id: Option<String>,

    #[clap(long = "leafchain-node-websocket-endpoint", help = "Leafchain node WebSocket endpoint")]
    pub leafchain_node_websocket_endpoint: Option<http::Uri>,

    #[clap(long = "allow-loopback-ip", help = "Allow to make connection with loopback IP address")]
    pub allow_loopback_ip: bool,

    #[clap(
        long = "exposed-rootchain-p2p-port",
        help = "Port for exposing the P2P network port of Rootchain"
    )]
    pub exposed_rootchain_p2p_port: Option<u16>,

    #[clap(
        long = "exposed-leafchain-p2p-port",
        help = "Port for exposing the P2P network port of Leafchain"
    )]
    pub exposed_leafchain_p2p_port: Option<u16>,
}
