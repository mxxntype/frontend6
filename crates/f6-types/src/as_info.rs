use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[must_use]
pub struct ASInfo {
    pub holder: Option<String>,
    pub domains: Vec<(IpAddr, String)>,
}
