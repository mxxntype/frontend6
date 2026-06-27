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
    pub infrastructure_groups: Vec<InfrastructureGroup>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[must_use]
pub struct InfrastructureGroup {
    pub asn: Option<u32>,
    pub as_holder: Option<String>,
    pub prefix: Option<String>,
    pub netname: Option<String>,
    pub description: Option<String>,
    pub country: Option<String>,
    pub maintainer: Option<String>,
    pub kind: InfrastructureKind,
    pub reason: String,
    pub ip_addrs: Vec<IpAddr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[must_use]
pub enum InfrastructureKind {
    Own,
    Hosting,
    Unknown,
}
