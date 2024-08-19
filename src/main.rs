#![allow(unused_imports)]
use anyhow::Result;
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
    /// List all hostdata from given database
    Ips {
        #[clap(short, long, default_value = "test_loglook")]
        /// Name of database to read from
        dbname: String,
    },
    /// List logentries associated with given ip address
    Logs {
        #[clap(short, long, default_value = "test_loglook")]
        /// Name of database to read from
        dbname: String,
        /// ip to look up
        #[clap(short, long)]
        ip: String,
    },
}

fn cmdprint(slug: &str) -> Result<()> {
    println!("{}", slug);
    Ok(())
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let cli = App::parse();
    let result = match &cli.command {
        Command::Ips { dbname } => {
            ipfox::get_ips_in_hostdata(dbname).await.unwrap();
            let slug = "The db is: ".to_owned() + dbname;
            cmdprint(&slug)
        }
        Command::Logs { dbname, ip } => {
            // print logentries for given ip
            ipfox::get_les_for_ip(dbname, ip).await.unwrap();
            Ok(())
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
