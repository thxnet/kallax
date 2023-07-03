use std::path::PathBuf;

use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Error occurs while writing file `{}`, error: {source}", path.display()))]
    WriteFile { path: PathBuf, source: std::io::Error },

    #[snafu(display("Error occurs while generating key pair from phrase, error: {source}"))]
    GenerateKeyPairFromPhrase { source: sp_core::crypto::SecretStringError },
}
