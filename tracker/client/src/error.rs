use snafu::{Backtrace, Snafu};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Error occurs while connecting to tracker, error: {source}"))]
    ConnectToTrackerGrpc { source: tonic::transport::Error, backtrace: Backtrace },
}
