use anyhow::{anyhow, Result};
use console::style;
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use mongodb::options::{FindOptions, IndexOptions};
use mongodb::{Client, Collection, Database, IndexModel};
use std::collections::HashSet;

pub mod geo;

pub mod hostdata;
pub mod logentries;

use hostdata::{Count, HostData};
use logentries::LogEntry;

/// Get database from dbname, return error if db doesn't exist
async fn get_db(dbname: &str) -> Result<Database> {
    let client = Client::with_uri_str("mongodb://192.168.0.128:27017").await?;
    let dbnames = client.list_database_names().await?;
    let has_db = dbnames.contains(&dbname.to_owned());
    if has_db {
        let db = client.database(dbname);
        Ok(db)
    } else {
        // bail if db doesn't exist
        Err(anyhow!("Database '{}' not found", dbname))
    }
}

/// get hostdata collection
async fn get_hostdata_coll(db: &Database) -> Result<Collection<HostData>> {
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
pub async fn get_les_for_ip(dbname: &str, count: &u32, ip: &str, nologs: &bool) -> Result<()> {
    let db = get_db(dbname).await?;
    let hostdata_coll = get_hostdata_coll(&db).await?;
    let logentries_coll = get_logentries_coll(&db).await?;
    let filter = doc! {"ip": ip};
    let ndisp: i64 = i64::from(*count);
    let fo = FindOptions::builder().limit(ndisp).build();
    if let Some(hostdata) = hostdata_coll.find_one(filter.clone()).await? {
        let cursor = logentries_coll.find(filter).with_options(fo).await?;
        let les: Vec<LogEntry> = cursor.try_collect().await?;
        println!(
            "Showing first {} logentries for ip {} in db {} ",
            les.len(),
            ip,
            dbname
        );

        println!("{}", hostdata);
        println!("-----------------");
        if !*nologs {
            // les.display_some(count)?;
            for le in les {
                println!("{}", le);
            }
            // output_les(&les, count).await?;
        }
        Ok(())
    } else {
        Err(anyhow!("No hostdata found for ip {}", ip))
    }
}

/// get count of logentries for each ip
pub async fn get_counts_by_ip(dbname: &str, maxlogs: &i64) -> Result<()> {
    let db = get_db(dbname).await?;
    let logentries_coll = get_logentries_coll(&db).await?;
    let hostdata_coll = get_hostdata_coll(&db).await?;
    let grouper = doc! {
        "$group": doc! {
            "_id": "$ip",
            "nles": doc! {
                "$sum": 1
            }
        }
    };
    let sorter = doc! {"$sort": doc! {"nles": -1}};
    let pipeline = vec![grouper, sorter];
    let mut cur = logentries_coll.aggregate(pipeline).await?;
    while let Some(doc) = cur.try_next().await? {
        let count: Count = bson::from_document(doc)?;
        let ip = count.ip.clone();
        let filter = doc! {"ip": count.ip};
        if let Some(hostdata) = hostdata_coll.find_one(filter.clone()).await? {
            println!("{}Count {}", hostdata, style(count.count).magenta());
            println!("-----------------\n");
        } else {
            println!("No hostdata found for ip {}", ip.clone());
        }
        // TODO: make limit a cmd line option
        let fo = FindOptions::builder().limit(*maxlogs).build();
        let le_cursor = logentries_coll
            .find(doc! {"ip": ip})
            .with_options(fo)
            .await?;
        let les: Vec<LogEntry> = le_cursor.try_collect().await?;
        for le in les {
            println!("{}", le);
        }
    }
    Ok(())
}

/// delete ips and associated logentries
pub async fn delete_ips(dbname: &str, ips: &Vec<String>) -> Result<()> {
    let db = get_db(dbname).await?;
    let hostdata_coll = get_hostdata_coll(&db).await?;
    let logentries_coll = get_logentries_coll(&db).await?;
    println!("Deleting ips: {:?}", ips);
    for ip in ips {
        let filter = doc! {"ip": ip};
        hostdata_coll.delete_one(filter.clone()).await?;
        logentries_coll.delete_many(filter).await?;
    }
    Ok(())
}

/// delete uptime probe logentries
pub async fn delete_probes(dbname: &str) -> Result<()> {
    // set of unique ips with DigitalOcean Uptime Probe as user agent
    let mut probe_ips = HashSet::new();
    let db = get_db(dbname).await?;
    let logentries_coll = get_logentries_coll(&db).await?;
    let hostdata_coll = get_hostdata_coll(&db).await?;
    let re = mongodb::bson::Regex {
        pattern: "^DigitalOcean Uptime Probe .*".to_string(),
        options: "i".to_string(),
    };
    let filter = doc! {"ua": re};
    let le_cursor = logentries_coll.find(filter).await?;
    let les: Vec<LogEntry> = le_cursor.try_collect().await?;
    for le in les {
        probe_ips.insert(le.ip);
    }
    for probe_ip in probe_ips {
        println!("Deleting probes associated with ip {}", probe_ip);
        let deleted_les = logentries_coll.delete_many(doc! {"ip": &probe_ip}).await?;
        let deleted_hds = hostdata_coll.delete_one(doc! {"ip": &probe_ip}).await?;
        println!(
            "Deleted {} logentries and {} hostdata",
            deleted_les.deleted_count, deleted_hds.deleted_count
        );
    }
    Ok(())
}
