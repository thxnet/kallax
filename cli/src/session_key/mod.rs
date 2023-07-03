mod config;
mod error;

pub use self::{
    config::Config,
    error::{Error, Result},
};

pub async fn run(config: Config) -> Result<()> {
    let Config { keystore_directory_path, session_key_mnemonic_phrase, node_name } = config;

    kallax_initializer::prepare_session_keys(
        keystore_directory_path,
        session_key_mnemonic_phrase,
        node_name,
    )
    .await?;

    Ok(())
}
