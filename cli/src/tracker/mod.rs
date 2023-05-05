mod config;
mod error;

use std::{net::SocketAddr, path::Path};

use kallax_primitives::ChainSpec;

use self::error::Result;
pub use self::{config::Config, error::Error};

async fn load_chain_spec_files<C, P>(chain_spec_files: C) -> Vec<ChainSpec>
where
    C: Iterator<Item = P>,
    P: AsRef<Path>,
{
    let mut chain_specs = Vec::new();

    for file in chain_spec_files {
        let file = file.as_ref();
        tracing::info!("Loading chain spec file `{}`", file.display());

        let content = match tokio::fs::read(&file).await {
            Ok(content) => content,
            Err(err) => {
                tracing::error!("Failed to load file `{}`, error: {err}", file.display());
                continue;
            }
        };

        let spec = match ChainSpec::try_from(content.as_ref()) {
            Ok(spec) => spec,
            Err(err) => {
                tracing::error!("Failed to parse chain spec `{}`, error: {err}", file.display());
                continue;
            }
        };

        chain_specs.push(spec);
        tracing::info!("Chain spec file `{}` loaded", file.display());
    }

    chain_specs
}

pub async fn run(config: Config) -> Result<()> {
    let Config { listen_address, listen_port, rootchain_spec_files, leafchain_spec_files } = config;
    let config = {
        let listen_address = SocketAddr::from((listen_address, listen_port));
        kallax_tracker_server::Config { listen_address }
    };

    let rootchain_specs = load_chain_spec_files(rootchain_spec_files.iter()).await;
    let leafchain_specs = load_chain_spec_files(leafchain_spec_files.iter()).await;

    kallax_tracker_server::start_grpc_server(config, rootchain_specs, leafchain_specs)
        .await
        .map_err(Error::from)
}
