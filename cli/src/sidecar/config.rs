use clap::Args;

#[derive(Args, Debug)]
pub struct Config {
    #[clap(long = "tracker-grpc-endpoint", help = "Tracker gRPC endpoint")]
    pub tracker_grpc_endpoint: http::Uri,

    #[clap(long = "rootchain-name", help = "Rootchain name")]
    pub rootchain_name: String,

    #[clap(long = "rootchain-node-websocket-endpoint", help = "Rootchain node WebSocket endpoint")]
    pub rootchain_node_websocket_endpoint: http::Uri,

    #[clap(long = "Leafchain name", help = "leafchain-name")]
    pub leafchain_name: Option<String>,

    #[clap(long = "leafchain-node-websocket-endpoint", help = "Leafchain node WebSocket endpoint")]
    pub leafchain_node_websocket_endpoint: Option<http::Uri>,
}
