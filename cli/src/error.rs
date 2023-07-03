use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not initialize tokio runtime, error: {source}"))]
    InitializeTokioRuntime { source: tokio::io::Error },

    #[snafu(display("{source}"))]
    SessionKey { source: crate::session_key::Error },

    #[snafu(display("{source}"))]
    Initializer { source: crate::initializer::Error },

    #[snafu(display("{source}"))]
    Sidecar { source: crate::sidecar::Error },

    #[snafu(display("{source}"))]
    Tracker { source: crate::tracker::Error },
}

impl From<crate::session_key::Error> for Error {
    fn from(source: crate::session_key::Error) -> Self { Self::SessionKey { source } }
}

impl From<crate::initializer::Error> for Error {
    fn from(source: crate::initializer::Error) -> Self { Self::Initializer { source } }
}

impl From<crate::sidecar::Error> for Error {
    fn from(source: crate::sidecar::Error) -> Self { Self::Sidecar { source } }
}

impl From<crate::tracker::Error> for Error {
    fn from(source: crate::tracker::Error) -> Self { Self::Tracker { source } }
}

pub trait CommandError {
    fn exit_code(&self) -> exitcode::ExitCode;
}

impl CommandError for Error {
    fn exit_code(&self) -> exitcode::ExitCode {
        match self {
            Self::InitializeTokioRuntime { .. } => exitcode::IOERR,
            Self::SessionKey { source } => source.exit_code(),
            Self::Initializer { source } => source.exit_code(),
            Self::Sidecar { source } => source.exit_code(),
            Self::Tracker { source } => source.exit_code(),
        }
    }
}
