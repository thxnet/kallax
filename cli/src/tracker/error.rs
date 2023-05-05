use snafu::Snafu;

use crate::error::CommandError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{source}"))]
    Application { source: kallax_tracker_server::Error },
}

impl From<kallax_tracker_server::Error> for Error {
    #[inline]
    fn from(source: kallax_tracker_server::Error) -> Self { Self::Application { source } }
}

impl CommandError for Error {
    fn exit_code(&self) -> exitcode::ExitCode {
        match self {
            Self::Application { .. } => exitcode::SOFTWARE,
        }
    }
}
