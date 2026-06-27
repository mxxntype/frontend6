use std::collections::HashMap;
use std::net::IpAddr;

use f6_types::as_info::ASInfo;

#[tracing::instrument(ret)]
pub async fn lookup_ip_asn(ip: &IpAddr) -> Option<Vec<u64>> {
    let url = format!("https://stat.ripe.net/data/network-info/data.json?resource={ip}");
    reqwest::get(url)
        .await
        .ok()?
        .json::<serde_json::Value>()
        .await
        .inspect(|json| tracing::debug!(?json))
        .ok()?
        .pointer("/data/asns")?
        .as_array()?
        .clone()
        .into_iter()
        .map(|asn| asn.as_str().and_then(|asn| asn.parse().ok()))
        .collect()
}

#[tracing::instrument(ret)]
pub async fn lookup_asn_holder(asn: u64) -> Option<String> {
    let url = format!("https://stat.ripe.net/data/as-overview/data.json?resource=AS{asn}");
    let holder = reqwest::get(url)
        .await
        .ok()?
        .json::<serde_json::Value>()
        .await
        .ok()?
        .pointer("/data/holder")?
        .as_str()?
        .to_string();
    Some(holder)
}

pub async fn group_as_info(ip_domain_sets: Vec<(String, IpAddr)>) -> HashMap<u64, ASInfo> {
    let mut map = HashMap::<u64, ASInfo>::new();

    for (domain, ip) in ip_domain_sets {
        let asns = lookup_ip_asn(&ip).await.unwrap_or_default();
        for asn in asns {
            let holder = lookup_asn_holder(asn).await;
            map.entry(asn)
                .and_modify(|as_info| as_info.domains.push((ip, domain.clone())))
                .or_insert_with(|| ASInfo {
                    holder,
                    domains: vec![(ip, domain.clone())],
                });
        }
    }

    map
}
