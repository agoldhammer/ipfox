use anyhow::Result;
use console::style;
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use mongodb::options::IndexOptions;
use mongodb::{Client, Collection, Database, IndexModel};
// use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod geo;
pub mod logentries;

use logentries::LogEntry;

#[derive(Debug, Serialize, Deserialize)]
pub struct HostData {
    pub ip: String,
    pub geodata: geo::Geodata,
    pub ptr_records: Vec<String>,
}

impl fmt::Display for HostData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "{}: {}",
            style("IP").bold().red(),
            style(&self.ip).green()
        )?;

        write!(f, "{}", self.geodata)?;
        self.ptr_records.iter().try_for_each(|record| {
            writeln!(f, "{}: {}", style("host").red(), style(record).green())
        })
    }
}

async fn get_db(dbname: &str) -> Result<Database> {
    let client = Client::with_uri_str("mongodb://192.168.0.128:27017").await?;
    let db = client.database(dbname);
    Ok(db)
}

async fn get_hostdata_coll(dbname: &str) -> Result<Collection<HostData>> {
    let db = get_db(dbname).await?;
    let collection = db.collection("hostdata");
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "ip": 1 })
        .options(options)
        .build();
    collection.create_index(model).await?;
    Ok(collection)
}

async fn get_logentries_coll(dbname: &str) -> Result<Collection<LogEntry>> {
    let db = get_db(dbname).await?;
    let collection = db.collection("logentries");
    let model = IndexModel::builder().keys(doc! {"ip": 1}).build();
    collection.create_index(model).await?;
    Ok(collection)
}

pub async fn get_ips_in_hostdata(dbname: &str) -> Result<()> {
    let hostdata_coll = get_hostdata_coll(dbname).await?;
    let alldocs = doc! {};
    let cursor = hostdata_coll.find(alldocs).await?;
    let hds: Vec<HostData> = cursor.try_collect().await?;
    for hd in hds {
        println!("{}", hd);
    }
    // while let Some(doc) = cursor.try_next().await? {
    //     println!("{:?}", doc);
    // }
    let count = hostdata_coll.estimated_document_count().await?;
    println!("count: {}", count);
    Ok(())
}

pub async fn get_les_for_ip(dbname: &str, ip: &str) -> Result<()> {
    let logentries_coll = get_logentries_coll(dbname).await?;
    let filter = doc! {"ip": ip};
    let cursor = logentries_coll.find(filter).await?;
    let les: Vec<LogEntry> = cursor.try_collect().await?;
    println!(
        "Showing {} logentries for db {} and ip {}",
        les.len(),
        dbname,
        ip
    );
    println!("-----------------");
    for le in les {
        println!("{}", le);
    }
    let count = logentries_coll.estimated_document_count().await?;
    println!("Total logentries in this db: {}", count);
    Ok(())
}
