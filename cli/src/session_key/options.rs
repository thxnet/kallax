use std::path::PathBuf;

use clap::Args;

use crate::consts;

#[derive(Args, Debug)]
pub struct Options {
    #[clap(long = "keystore-directory-path", help = "Keystore directory path")]
    pub keystore_directory_path: PathBuf,

    #[clap(
        long = "session-key-mnemonic-phrase",
        env = consts::KALLAX_SESSION_KEY_MNEMONIC_PHRASE_ENV,
        help = "Session key mnemonic phrase"
    )]
    pub session_key_mnemonic_phrase: String,

    #[clap(long = "node-name", help = "Node name")]
    pub node_name: String,
}
