use snafu::Snafu;

use crate::error::CommandError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{source}"))]
    Application { source: kallax_tracker_server::Error },

    #[snafu(display("{source}"))]
    JoinTaskHandle { source: tokio::task::JoinError },

    #[snafu(display("Error occurs while creating UNIX signal listener, error: {source}"))]
    CreateUnixSignalListener { source: std::io::Error },
}

impl From<kallax_tracker_server::Error> for Error {
    #[inline]
    fn from(source: kallax_tracker_server::Error) -> Self { Self::Application { source } }
}

impl CommandError for Error {
    fn exit_code(&self) -> exitcode::ExitCode {
        match self {
            Self::Application { .. } => exitcode::SOFTWARE,
            Self::JoinTaskHandle { .. } | Self::CreateUnixSignalListener { .. } => exitcode::IOERR,
        }
    }
}
