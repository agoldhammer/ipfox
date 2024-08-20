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

async fn get_ips_in_hostdata(dbname: &str) -> Result<Vec<String>> {
    let hostdata_coll = get_hostdata_coll(dbname).await?;
    let alldocs = doc! {};
    let cursor = hostdata_coll.find(alldocs).await?;
    let hds: Vec<HostData> = cursor.try_collect().await?;
    let mut ips = hds.iter().map(|hd| hd.ip.clone()).collect::<Vec<_>>();
    ips.sort();
    Ok(ips)
}

pub async fn list_ips_in_hostdata(dbname: &str) -> Result<()> {
    let ips = get_ips_in_hostdata(dbname).await?;
    println!("ips: {:?}", ips);
    println!("ips found: {}", ips.len());
    // for hd in hds {
    //     println!("{}", hd);
    // }
    // while let Some(doc) = cursor.try_next().await? {
    //     println!("{:?}", doc);
    // }
    // let count = hostdata_coll.estimated_document_count().await?;
    // println!("count: {}", count);
    Ok(())
}

pub async fn output_hostdata_by_ip(dbname: &str) -> Result<()> {
    let ips = get_ips_in_hostdata(dbname).await?;
    let db = get_db(dbname).await?;
    let logentries_coll: Collection<LogEntry> = db.collection("logentries");
    for ip in ips {
        let filter = doc! {"ip": ip};
        let cursor = logentries_coll.find(filter).await?;
        let les: Vec<LogEntry> = cursor.try_collect().await?;
        for le in les {
            println!("{}", le);
        }
    }
    Ok(())
}

// async fn get_cursor_of_les(coll: Collection<LogEntry>, ip: &str) -> Result<Cursor<LogEntry>> {
//     let filter = doc! {"ip": ip};
//     let cursor = coll.find(filter).await?;
//     Ok(cursor)
// }

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
    // let count = logentries_coll.estimated_document_count().await?;
    // println!("Total logentries in this db: {}", count);
    Ok(())
}
