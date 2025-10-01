use std::path::PathBuf;

use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{source}"))]
    NodeKey { source: crate::node_key::Error },

    #[snafu(display("{source}"))]
    SessionKey { source: crate::session_key::Error },

    #[snafu(display("{source}"))]
    Tracker { source: kallax_tracker_grpc_client::Error },

    #[snafu(display("Error occurs while creating directory `{}`, error: {source}", path.display()))]
    CreateDirectory { path: PathBuf, source: std::io::Error },

    #[snafu(display("Error occurs while writing file `{}`, error: {source}", path.display()))]
    WriteFile { path: PathBuf, source: std::io::Error },

    #[snafu(display("{error_message}"))]
    GetChainSpec { error_message: String },
}

impl From<crate::node_key::Error> for Error {
    fn from(source: crate::node_key::Error) -> Self {
        Self::NodeKey { source }
    }
}

impl From<crate::session_key::Error> for Error {
    fn from(source: crate::session_key::Error) -> Self {
        Self::SessionKey { source }
    }
}

impl From<kallax_tracker_grpc_client::Error> for Error {
    fn from(source: kallax_tracker_grpc_client::Error) -> Self {
        Self::Tracker { source }
    }
}
