mod config;
mod error;

pub use self::{
    config::Config,
    error::{Error, Result},
};

pub async fn run(config: Config) -> Result<()> {
    let config = {
        let Config {
            node_key_file_path,
            tracker_grpc_endpoint,
            rootchain_id,
            rootchain_spec_file_path,
            leafchain_id,
            leafchain_spec_file_path,
            keystore_directory_path,
            session_key_mnemonic_phrase,
            node_name,
        } = config;
        kallax_initializer::Config {
            node_key_file_path,
            tracker_grpc_endpoint,
            rootchain_id,
            rootchain_spec_file_path,
            leafchain_id,
            leafchain_spec_file_path,
            keystore_directory_path,
            session_key_mnemonic_phrase,
            node_name,
        }
    };

    kallax_initializer::prepare(config).await?;

    Ok(())
}
