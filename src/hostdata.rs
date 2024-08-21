use crate::geo;
use console::style;
use serde::{Deserialize, Serialize};
use std::fmt;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Count {
    #[serde(rename = "_id")]
    pub ip: String,
    #[serde(rename = "nles")]
    pub count: u32,
}
