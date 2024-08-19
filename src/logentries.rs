use bson;
use serde::{Deserialize, Serialize};
use std::fmt;

// #[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub ip: String,
    pub time: bson::DateTime,
    pub method: String,
    pub code: u32,
    pub nbytes: u32,
    pub referrer: String,
    pub ua: String,
    pub line: String,
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "  ip: {}\n", self.ip)?;
        writeln!(f, "  time: {}", self.time)?;
        writeln!(f, "  method: {}", self.method)?;
        writeln!(f, "  code: {}", self.code)?;
        writeln!(f, "  nbytes: {}", self.nbytes)?;
        writeln!(f, "  referrer: {}", self.referrer)?;
        writeln!(f, "  user agent: {}", self.ua)?;
        writeln!(f, "  logged: {}:", self.line)?;
        writeln!(f, "end")
    }
}
