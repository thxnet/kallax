mod error;
mod options;

pub use self::{
    error::{Error, Result},
    options::Options,
};

pub async fn run(options: Options) -> Result<()> {
    let options = {
        let Options {
            node_key_file_path,
            tracker_grpc_endpoint,
            rootchain_id,
            rootchain_spec_file_path,
            leafchain_id,
            leafchain_spec_file_path,
            keystore_directory_path,
            session_key_mnemonic_phrase,
            node_name,
        } = options;
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

    kallax_initializer::prepare(options).await?;

    Ok(())
}
