use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use std::ops::RangeInclusive;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Connect {
        #[arg(default_value = "127.0.0.1")]
        host: String,
        #[arg(short, long, value_parser = port_in_range, default_value = "9090")]
        port: u16,
        #[arg(long)]
        rpc: String,
    },
    Serve {
        #[arg(default_value = "127.0.0.1")]
        host: String,
        #[arg(short, long, value_parser = port_in_range, default_value = "9090")]
        port: u16,
        #[arg(short, long, default_value = "false")]
        discovery: bool,
    },
}
fn port_in_range(s: &str) -> Result<u16> {
    const PORT_RANGE: RangeInclusive<usize> = 1..=65535;
    let port: usize = s
        .parse()
        .with_context(|| format!("`{}` is not a valid port number", s))?;
    if PORT_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(anyhow!("port out of range"))
    }
}
