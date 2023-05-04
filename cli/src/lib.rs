mod consts;
mod error;
mod initializer;
mod sidecar;
mod tracker;

use std::{fmt, future::Future, io::Write};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use snafu::ResultExt;
use tokio::runtime;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use self::error::{CommandError, Error, Result};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

impl Default for Cli {
    #[inline]
    fn default() -> Self { Self::parse() }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Show current version")]
    Version,

    #[command(about = "Show shell completions")]
    Completions { shell: Shell },

    #[command(about = "Run initializer for starting Substrate-based node")]
    Initializer {
        #[clap(flatten)]
        config: initializer::Config,
    },

    #[command(about = "Run Kubernetes sidecar for Substrate-based node")]
    Sidecar {
        #[clap(flatten)]
        config: sidecar::Config,
    },

    #[command(about = "Run tracker for Substrate-based node")]
    Tracker {
        #[clap(flatten)]
        config: tracker::Config,
    },
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.commands {
            Commands::Version => {
                let mut stdout = std::io::stdout();
                stdout
                    .write_all(Self::command().render_long_version().as_bytes())
                    .expect("failed to write to stdout");
                Ok(())
            }
            Commands::Completions { shell } => {
                let mut app = Self::command();
                let bin_name = app.get_name().to_string();
                clap_complete::generate(shell, &mut app, bin_name, &mut std::io::stdout());
                Ok(())
            }
            Commands::Initializer { config } => {
                execute("Initializer", async { initializer::run(config).await })
            }
            Commands::Sidecar { config } => {
                execute("Sidecar", async { sidecar::run(config).await })
            }
            Commands::Tracker { config } => {
                execute("Tracker", async { tracker::run(config).await })
            }
        }
    }
}

#[inline]
fn execute<S, F, E>(command_name: S, fut: F) -> Result<()>
where
    S: fmt::Display,
    F: Future<Output = std::result::Result<(), E>>,
    E: Into<Error>,
{
    init_tracing();

    tracing::info!("Starting {}", Cli::command().get_long_version().unwrap_or_default());
    tracing::info!("Run {command_name}");

    tracing::info!("Initializing Tokio runtime");
    let runtime = runtime::Builder::new_multi_thread()
        .thread_name(consts::THREAD_NAME)
        .enable_all()
        .build()
        .context(error::InitializeTokioRuntimeSnafu)?;

    runtime.block_on(fut).map_err(Into::into)
}

fn init_tracing() {
    // filter
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    // format
    let fmt_layer =
        tracing_subscriber::fmt::layer().pretty().with_thread_ids(true).with_thread_names(true);
    // subscriber
    tracing_subscriber::registry().with(filter_layer).with(fmt_layer).init();
}
