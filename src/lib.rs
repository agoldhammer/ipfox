use anyhow::anyhow;
use anyhow::Result;
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use mongodb::options::IndexOptions;
use mongodb::{Client, Collection, Database, IndexModel};

pub mod geo;

pub(crate) mod hostdata;
pub(crate) mod logentries;

use hostdata::HostData;
use logentries::LogEntry;

/// Get database from dbname, return error if db doesn't exist
async fn get_db(dbname: &str) -> Result<Database> {
    let client = Client::with_uri_str("mongodb://192.168.0.128:27017").await?;
    let mut dbnames = client.list_database_names().await?;
    dbnames.sort();
    // check to see if desired db is in the list of names
    let has_db = &dbnames.binary_search(&dbname.to_owned());
    match has_db {
        Ok(_) => {
            let db = client.database(dbname);
            Ok(db)
        }
        // bail if db doesn't exist
        Err(_) => Err(anyhow!("Database '{}' not found", dbname)),
    }
}

/// get hostdata collection
async fn get_hostdata_coll(db: &Database) -> Result<Collection<HostData>> {
    // let db = get_db(dbname).await?;
    let collection = db.collection("hostdata");
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "ip": 1 })
        .options(options)
        .build();
    collection.create_index(model).await?;
    Ok(collection)
}

/// get logentries collection
async fn get_logentries_coll(db: &Database) -> Result<Collection<LogEntry>> {
    let collection = db.collection("logentries");
    let model = IndexModel::builder().keys(doc! {"ip": 1}).build();
    collection.create_index(model).await?;
    Ok(collection)
}

/// get sorted list of ips in hostdata
async fn get_ips_in_hostdata(dbname: &str) -> Result<Vec<String>> {
    let db = get_db(dbname).await?;
    let hostdata_coll = get_hostdata_coll(&db).await?;
    let alldocs = doc! {};
    let cursor = hostdata_coll.find(alldocs).await?;
    let hds: Vec<HostData> = cursor.try_collect().await?;
    let mut ips = hds.iter().map(|hd| hd.ip.clone()).collect::<Vec<_>>();
    ips.sort();
    Ok(ips)
}

/// output flat list of all ips in hostdata
pub async fn list_ips_in_hostdata(dbname: &str) -> Result<()> {
    let ips = get_ips_in_hostdata(dbname).await?;
    println!("ips: {:?}", ips);
    println!("ips found: {}", ips.len());
    Ok(())
}

/// output all hostdata, grouped by ip
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

/// get all logentries for given ip
pub async fn get_les_for_ip(dbname: &str, ip: &str) -> Result<()> {
    let db = get_db(dbname).await?;
    let logentries_coll = get_logentries_coll(&db).await?;
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

// aggregation needed
// [
//     doc! {
//         "$group": doc! {
//             "_id": "$ip",
//             "nles": doc! {
//                 "$sum": 1
//             }
//         }
//     }
// ]

// let grouper = doc! {"$group": {"_id": "$ip"}};
//     let sorter = doc! {"$sort": {"_id": 1}};
//     let pipeline = vec![time_filter, grouper, sorter];
//     coll.aggregate(pipeline, None)
//         .await
//         .map_err(anyhow::Error::msg)
