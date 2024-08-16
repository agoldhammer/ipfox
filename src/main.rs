#![allow(unused_imports)]
use std::process;

use clap::{ArgAction, Parser, Subcommand};

// * https://rust-cli-recommendations.sunshowers.io/handling-arguments.html
#[derive(Debug, Parser)]
#[clap(name = "ipfox", version = "0.1", about = "Retrieve ips")]
pub struct App {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Read from database
    Read {
        #[clap(long)]
        dbname: String,
    },
}

fn cmdprint(slug: &str) -> anyhow::Result<()> {
    println!("{}", slug);
    Ok(())
}

fn main() {
    println!("Hello, world!");
    let cli = App::parse();
    let result = match &cli.command {
        Command::Read { dbname } => {
            let slug = "The db is: ".to_owned() + dbname;
            cmdprint(&slug)
        }
    };

    match result {
        Ok(()) => {
            println!("Normal exit!");
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Application error: {}", e);
            process::exit(1);
        }
    }
}
