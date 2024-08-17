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

// impl Geodata {
//     fn new(ip: &str) -> Geodata {
//         Geodata {
//             ip: String::from(ip),
//             country_name: "".to_string(),
//             state_prov: "".to_string(),
//             city: "".to_string(),
//             isp: "".to_string(),
//             organization: "".to_string(),
//         }
//     }
// }

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
