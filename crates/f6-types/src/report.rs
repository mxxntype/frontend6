use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::LegalEntityTIN;
use crate::as_info::ASInfo;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[must_use]
pub struct TINReport {
    pub tin: LegalEntityTIN,
    pub name: String,
    pub domains: HashSet<String>,
    pub ip_addrs: HashMap<String, IpAddr>,
    pub ripe_info: HashMap<u64, ASInfo>,
}
