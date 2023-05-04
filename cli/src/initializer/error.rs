use snafu::Snafu;

use crate::error::CommandError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {}

impl CommandError for Error {
    fn exit_code(&self) -> exitcode::ExitCode {
        // TODO: use proper exit code
        exitcode::USAGE
    }
}
