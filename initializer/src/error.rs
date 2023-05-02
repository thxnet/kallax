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
    Tracker { source: kallax_tracker_client::Error },

    #[snafu(display("Error occurs while creating directory `{}`, error: {source}", path.display()))]
    CreateDirectory { path: PathBuf, source: std::io::Error },

    #[snafu(display("Error occurs while writing file `{}`, error: {source}", path.display()))]
    WriteFile { path: PathBuf, source: std::io::Error },

    #[snafu(display("{source}"))]
    GetChainSpec { source: kallax_tracker_client::chain_spec::Error },
}

impl From<crate::node_key::Error> for Error {
    fn from(source: crate::node_key::Error) -> Self { Error::NodeKey { source } }
}

impl From<crate::session_key::Error> for Error {
    fn from(source: crate::session_key::Error) -> Self { Error::SessionKey { source } }
}

impl From<kallax_tracker_client::Error> for Error {
    fn from(source: kallax_tracker_client::Error) -> Self { Error::Tracker { source } }
}
