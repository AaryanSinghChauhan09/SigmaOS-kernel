use clap::Parser;
use kernel::kernel;

#[derive(Parser)]
struct Cli;

fn main() -> anyhow::Result<()> {
    let config = kernel::Config::default();
    let mut component = kernel::new(config);
    component.initialize()?;
    println!("SigmaOS-kernel ready");
    Ok(())
}
