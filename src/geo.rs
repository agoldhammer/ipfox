use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct Geodata {
    pub ip: String,
    pub country_name: String,
    pub state_prov: String,
    pub city: String,
    pub isp: String,
    pub organization: String,
}

impl fmt::Display for Geodata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Loc: {}, {}, {}",
            self.city, self.state_prov, self.country_name
        )?;
        writeln!(f, "ISP: {}", self.isp)?;
        writeln!(f, "Org: {}", self.organization)
    }
}
