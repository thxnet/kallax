use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Error occurs while connecting tracker: `{uri}`, error: {source}"))]
    ConnectTracker { uri: http::Uri, source: kallax_tracker_grpc_client::Error },

    #[snafu(display(
        "Error occurs while connecting Substrate-based node: `{uri}`, error: {error}"
    ))]
    ConnectSubstrateNode { uri: http::Uri, error: String },

    #[snafu(display(
        "Error occurs while fetching local listen address from Substrate-based node, error: \
         {source}"
    ))]
    FetchLocalListenAddressesFromSubstrateNode { source: substrate_rpc_client::Error },

    #[snafu(display(
        "Error occurs while fetching peers from Substrate-based node, error: {source}"
    ))]
    FetchPeersFromSubstrateNode { source: substrate_rpc_client::Error },

    #[snafu(display("Error occurs while fetching peer addresses from Tracker, error: {source}"))]
    GetPeerAddressesFromTracker { source: kallax_tracker_grpc_client::Error },
}
