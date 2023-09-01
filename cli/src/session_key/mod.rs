mod error;
mod options;

pub use self::{
    error::{Error, Result},
    options::Options,
};

pub async fn run(options: Options) -> Result<()> {
    let Options { keystore_directory_path, session_key_mnemonic_phrase, node_name } = options;

    kallax_initializer::prepare_session_keys(
        keystore_directory_path,
        session_key_mnemonic_phrase,
        node_name,
    )
    .await?;

    Ok(())
}
