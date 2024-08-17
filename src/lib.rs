use mongodb::bson::doc;
use mongodb::options::IndexOptions;
use mongodb::{Client, Collection, IndexModel};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Ip {
    pub ip: String,
    pub timestamp: i64,
}
