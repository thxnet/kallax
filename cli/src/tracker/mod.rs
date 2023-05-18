mod config;
mod error;

use std::{net::SocketAddr, path::Path};

use futures::FutureExt;
use kallax_primitives::ChainSpec;
use snafu::ResultExt;
use tokio::signal::unix::{signal, SignalKind};

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
    let Config {
        listen_address,
        listen_port,
        rootchain_spec_files,
        leafchain_spec_files,
        allow_loopback_ip,
    } = config;
    let config = {
        let listen_address = SocketAddr::from((listen_address, listen_port));
        kallax_tracker_server::Config { listen_address, allow_loopback_ip }
    };

    let rootchain_specs = {
        let mut specs = load_chain_spec_files(rootchain_spec_files.iter()).await;
        specs.push(
            ChainSpec::try_from(include_bytes!("chain-specs/mainnet.rootchain.raw.json").as_ref())
                .expect("`mainnet.rootchain.raw.json` is a valid spec"),
        );
        specs.push(
            ChainSpec::try_from(include_bytes!("chain-specs/testnet.rootchain.raw.json").as_ref())
                .expect("`testnet.rootchain.raw.json` is a valid spec"),
        );
        specs
    };
    let leafchain_specs = {
        let mut specs = load_chain_spec_files(leafchain_spec_files.iter()).await;
        specs.push(
            ChainSpec::try_from(
                include_bytes!("chain-specs/testnet.leafchain.thx.raw.json").as_ref(),
            )
            .expect("`testnet.leafchain.thx.raw.json` is a valid spec"),
        );

        specs.push(
            ChainSpec::try_from(
                include_bytes!("chain-specs/testnet.leafchain.lmt.raw.json").as_ref(),
            )
            .expect("`testnet.leafchain.lmt.raw.json` is a valid spec"),
        );
        specs
    };

    let (shutdown_handle, tracker_server_handle) = {
        let (shutdown_handle, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();

        let server_handle = tokio::spawn(async move {
            let shutdown_signal = async move {
                rx.recv().await;
            }
            .boxed();

            kallax_tracker_server::serve_with_shutdown(
                config,
                rootchain_specs,
                leafchain_specs,
                shutdown_signal,
            )
            .await
            .map_err(Error::from)
        });

        (shutdown_handle, server_handle)
    };

    tracing::debug!("Create UNIX signal listener for `SIGTERM`");
    let mut sigterm =
        signal(SignalKind::terminate()).context(error::CreateUnixSignalListenerSnafu)?;
    tracing::debug!("Create UNIX signal listener for `SIGINT`");
    let mut sigint =
        signal(SignalKind::interrupt()).context(error::CreateUnixSignalListenerSnafu)?;

    tracing::debug!("Wait for shutdown signal");
    drop(futures::future::select(sigterm.recv().boxed(), sigint.recv().boxed()).await);

    // send shutdown signal to unbounded channel receiver by dropping the sender
    drop(shutdown_handle);

    tracker_server_handle.await.context(error::JoinTaskHandleSnafu)?
}
