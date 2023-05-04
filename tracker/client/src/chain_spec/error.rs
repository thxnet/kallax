use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{source}"))]
    Primitives { source: kallax_tracker_primitives::Error },

    #[snafu(display("Error occurs while get chain spec from Tracker, error: {}", source.message()))]
    GetChainSpec { source: tonic::Status },

    #[snafu(display("Missing chain spec field"))]
    MissingChainSpec,
}

impl From<kallax_tracker_primitives::Error> for Error {
    fn from(source: kallax_tracker_primitives::Error) -> Self { Self::Primitives { source } }
}
