mod config;
mod error;

pub use self::{
    config::Config,
    error::{Error, Result},
};

pub async fn run(_config: Config) -> Result<()> { Ok(()) }
