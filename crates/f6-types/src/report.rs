use std::collections::HashSet;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::LegalEntityTIN;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[must_use]
pub struct TINReport {
    pub tin: LegalEntityTIN,
    pub name: String,
    pub domains: HashSet<String>,
    pub ip_addrs: HashSet<IpAddr>,
}
