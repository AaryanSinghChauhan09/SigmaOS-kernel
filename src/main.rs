use std::env;
use log::{info, warn, error, debug};
use anyhow::Result;
use clap::Parser;
use kernel::kernel;

#[derive(Parser)]
struct Cli;

fn main() -> Result<()> { -> anyhow::Result<()> {
    let config = kernel::Config::default();
    let mut component = kernel::new(config);
    component.initialize()?;
    println!("SigmaOS-kernel ready");
    Ok(())
}

#[derive(Debug, Clone)]
struct Config {
    verbose: bool,
    dry_run: bool,
}

impl Config {
    fn from_env() -> Self {
        Config {
            verbose: env::var("VERBOSE").is_ok(),
            dry_run: env::var("DRY_RUN").is_ok(),
        }
    }
}
