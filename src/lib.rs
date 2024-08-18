use anyhow::Result;
use console::style;
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use mongodb::options::IndexOptions;
use mongodb::{Client, Collection, IndexModel};
// use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod geo;

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

async fn setup_db(dbname: &str) -> Result<Collection<HostData>> {
    let client = Client::with_uri_str("mongodb://192.168.0.128:27017").await?;
    let db = client.database(dbname);
    let collection = db.collection("hostdata");
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "ip": 1 })
        .options(options)
        .build();
    collection.create_index(model).await?;
    Ok(collection)
}

pub async fn get_ips_in_hostdata(dbname: &str) -> Result<()> {
    let hostdata_coll = setup_db(dbname).await?;
    let filter = doc! { "ip": { "$ne": "" } };
    let mut cursor = hostdata_coll.find(filter).await?;
    while let Some(doc) = cursor.try_next().await? {
        println!("{:?}", doc);
    }
    let count = hostdata_coll.estimated_document_count().await?;
    println!("count: {}", count);
    Ok(())
}
