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
    /// List unique ips for database
    Ips {
        #[clap(short, long, default_value = "test_loglook")]
        /// Name of database to read from
        dbname: String,
    },
    /// List hostdata and logentries associated with given ip address
    Logs {
        #[clap(short, long, default_value = "test_loglook")]
        /// Name of database to read from
        dbname: String,
        /// ip to look up
        #[clap(short, long)]
        ip: String,
        /// suppress logentry output
        #[clap(short, long, default_value = "false", action = ArgAction::SetTrue)]
        nologs: bool,
    },
    /// Output logentries for each ip in hostdata
    All {
        #[clap(short, long, default_value = "test_loglook")]
        /// Name of database to read from
        dbname: String,
    },
    /// Output count of logentries for each ip in hostdata
    Counts {
        #[clap(short, long, default_value = "test_loglook")]
        /// Name of database to read from
        dbname: String,
    },
    /// Delete ips and associated logentries
    Del {
        #[clap(short, long, default_value = "test_loglook")]
        /// Name of database to read from
        dbname: String,
        /// ips to delete
        ips: Vec<String>,
    },
}

// fn cmdprint(slug: &str) -> Result<()> {
//     println!("{}", slug);
//     Ok(())
// }

#[tokio::main(worker_threads = 8)]
async fn main() {
    let cli = App::parse();
    // let mut cmd_result: Result<()>;
    let result = match &cli.command {
        Command::Ips { dbname } => ipfox::list_ips_in_hostdata(dbname).await,
        Command::Logs { dbname, ip, nologs } => ipfox::get_les_for_ip(dbname, ip, nologs).await,
        Command::All { dbname } => ipfox::output_hostdata_by_ip(dbname).await,
        Command::Counts { dbname } => ipfox::get_counts_by_ip(dbname).await,
        Command::Del { dbname, ips } => ipfox::delete_ips(dbname, ips).await,
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
