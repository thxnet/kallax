use snafu::Snafu;

use crate::error::CommandError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{source}"))]
    Application { source: kallax_sidecar::Error },

    #[snafu(display("{source}"))]
    JoinTaskHandle { source: tokio::task::JoinError },

    #[snafu(display("Error occurs while creating UNIX signal listener, error: {source}"))]
    CreateUnixSignalListener { source: std::io::Error },

    #[snafu(display("Leafchain name must be provided"))]
    LeafchainNameNotProvided,

    #[snafu(display("Leafchain node WebSocket endpoint must be provided"))]
    LeafchainNodeWebSocketEndpointNotProvided,
}

impl From<kallax_sidecar::Error> for Error {
    #[inline]
    fn from(source: kallax_sidecar::Error) -> Self {
        Self::Application { source }
    }
}

impl CommandError for Error {
    fn exit_code(&self) -> exitcode::ExitCode {
        match self {
            Self::Application { .. } => exitcode::SOFTWARE,
            Self::JoinTaskHandle { .. } | Self::CreateUnixSignalListener { .. } => exitcode::IOERR,
            Self::LeafchainNameNotProvided | Self::LeafchainNodeWebSocketEndpointNotProvided => {
                exitcode::USAGE
            }
        }
    }
}
