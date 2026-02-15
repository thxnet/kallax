mod error;
mod options;

use std::{net::SocketAddr, path::Path, time::Duration};

use kallax_primitives::ChainSpec;

use self::error::Result;
pub use self::{error::Error, options::Options};

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

pub async fn run(options: Options) -> Result<()> {
    let Options {
        api_listen_address,
        api_listen_port,
        grpc_listen_address,
        grpc_listen_port,
        rootchain_spec_files,
        leafchain_spec_files,
        allow_peer_in_loopback_network,
        peer_time_to_live,
    } = options;
    let config = {
        let api_listen_address = SocketAddr::from((api_listen_address, api_listen_port));
        let grpc_listen_address = SocketAddr::from((grpc_listen_address, grpc_listen_port));
        let peer_time_to_live = Duration::from_secs(peer_time_to_live);
        kallax_tracker_server::Config {
            api_listen_address,
            grpc_listen_address,
            allow_peer_in_loopback_network,
            peer_time_to_live,
        }
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
        // chain_specs of testnet
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

        specs.push(
            ChainSpec::try_from(
                include_bytes!("chain-specs/testnet.leafchain.txd.raw.json").as_ref(),
            )
            .expect("`testnet.leafchain.txd.raw.json` is a valid spec"),
        );

        specs.push(
            ChainSpec::try_from(
                include_bytes!("chain-specs/testnet.leafchain.sand.raw.json").as_ref(),
            )
            .expect("`testnet.leafchain.sand.raw.json` is a valid spec"),
        );

        specs.push(
            ChainSpec::try_from(
                include_bytes!("chain-specs/testnet.leafchain.aether.raw.json").as_ref(),
            )
            .expect("`testnet.leafchain.aether.raw.json` is a valid spec"),
        );

        specs.push(
            ChainSpec::try_from(
                include_bytes!("chain-specs/testnet.leafchain.izutsuya.raw.json").as_ref(),
            )
            .expect("`testnet.leafchain.izutsuya.raw.json` is a valid spec"),
        );

        specs.push(
            ChainSpec::try_from(
                include_bytes!("chain-specs/testnet.leafchain.mirrored-body.raw.json").as_ref(),
            )
            .expect("`testnet.leafchain.mirrored-body.raw.json` is a valid spec"),
        );

        specs.push(
            ChainSpec::try_from(
                include_bytes!("chain-specs/testnet.leafchain.ecq.raw.json").as_ref(),
            )
            .expect("`testnet.leafchain.ecq.raw.json` is a valid spec"),
        );

        // chain_specs of mainnet
        specs.push(
            ChainSpec::try_from(
                include_bytes!("chain-specs/mainnet.leafchain.thx.raw.json").as_ref(),
            )
            .expect("`mainnet.leafchain.thx.raw.json` is a valid spec"),
        );

        specs.push(
            ChainSpec::try_from(
                include_bytes!("chain-specs/mainnet.leafchain.lmt.raw.json").as_ref(),
            )
            .expect("`mainnet.leafchain.lmt.raw.json` is a valid spec"),
        );

        specs.push(
            ChainSpec::try_from(
                include_bytes!("chain-specs/mainnet.leafchain.activa.raw.json").as_ref(),
            )
            .expect("`mainnet.leafchain.activa.raw.json` is a valid spec"),
        );

        specs.push(
            ChainSpec::try_from(
                include_bytes!("chain-specs/mainnet.leafchain.avatect.raw.json").as_ref(),
            )
            .expect("`mainnet.leafchain.avatect.raw.json` is a valid spec"),
        );

        specs.push(
            ChainSpec::try_from(
                include_bytes!("chain-specs/mainnet.leafchain.mirrored-body.raw.json").as_ref(),
            )
            .expect("`mainnet.leafchain.mirrored-body.raw.json` is a valid spec"),
        );

        specs.push(
            ChainSpec::try_from(
                include_bytes!("chain-specs/mainnet.leafchain.ecq.raw.json").as_ref(),
            )
            .expect("`mainnet.leafchain.ecq.raw.json` is a valid spec"),
        );

        specs
    };

    kallax_tracker_server::serve(config, rootchain_specs, leafchain_specs)
        .await
        .map_err(Error::from)?;

    Ok(())
}
