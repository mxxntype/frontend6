use std::collections::HashSet;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[must_use]
pub struct IpAddrResponse(pub HashSet<IpAddr>);
