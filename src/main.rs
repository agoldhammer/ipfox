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
        #[clap(short, long, default_value = "loglook")]
        /// Name of database to read from
        dbname: String,
    },
    /// List hostdata and logentries associated with given ip address
    Logs {
        /// Name of database to read from
        #[clap(short, long, default_value = "loglook")]
        dbname: String,

        /// number of logentries to display
        #[clap(short, long, default_value = "1")]
        count: u32,

        /// ip to look up
        #[clap(short, long)]
        ip: String,

        /// suppress logentry output
        #[clap(short, long, default_value = "false", action = ArgAction::SetTrue)]
        nologs: bool,
    },
    /// Output logentries for each ip in hostdata
    All {
        #[clap(short, long, default_value = "loglook")]
        /// Name of database to read from
        dbname: String,
    },
    /// Output count of logentries for each ip in hostdata
    Counts {
        #[clap(short, long, default_value = "loglook")]
        /// Name of database to read from
        dbname: String,
        #[clap(short, long, default_value = "2")]
        /// Maximum number of logentries to show for each ip
        maxlogs: i64,
    },
    /// Delete ips and associated logentries
    Del {
        #[clap(short, long, default_value = "loglook")]
        /// Name of database to read from
        dbname: String,
        /// ips to delete
        ips: Vec<String>,
    },
    /// Delete uptime probe logentries
    Probes {
        #[clap(short, long, default_value = "loglook")]
        /// Name of database to read from
        dbname: String,
    },
}

#[tokio::main(worker_threads = 8)]
async fn main() {
    console::set_colors_enabled(true);
    let cli = App::parse();
    let result = match &cli.command {
        Command::Ips { dbname } => ipfox::list_ips_in_hostdata(dbname).await,
        Command::Logs {
            dbname,
            count,
            ip,
            nologs,
        } => ipfox::get_les_for_ip(dbname, count, ip, nologs).await,
        Command::All { dbname } => ipfox::output_hostdata_by_ip(dbname).await,
        Command::Counts { dbname, maxlogs } => ipfox::get_counts_by_ip(dbname, maxlogs).await,
        Command::Del { dbname, ips } => ipfox::delete_ips(dbname, ips).await,
        Command::Probes { dbname } => ipfox::delete_probes(dbname).await,
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
