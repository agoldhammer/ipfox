use clap::{ArgAction, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name="ipfox", version="0.1", about="Retrieve ips")]
pub struct App {
    #[clap(subcommand)]
    command: Command,
}

fn main() {
    println!("Hello, world!");
}
