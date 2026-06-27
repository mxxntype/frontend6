use std::collections::{BTreeMap, HashSet};
use std::net::IpAddr;

use f6_types::report::{InfrastructureGroup, InfrastructureKind};
use serde::Deserialize;

const RIPESTAT_NETWORK_INFO_URL: &str = "https://stat.ripe.net/data/network-info/data.json";
const RIPESTAT_WHOIS_URL: &str = "https://stat.ripe.net/data/whois/data.json";
const RIPESTAT_AS_OVERVIEW_URL: &str = "https://stat.ripe.net/data/as-overview/data.json";

#[derive(Debug)]
struct IpMetadata {
    ip_addr: IpAddr,
    asn: Option<u32>,
    as_holder: Option<String>,
    prefix: Option<String>,
    netname: Option<String>,
    description: Option<String>,
    country: Option<String>,
    maintainer: Option<String>,
}

#[derive(Deserialize)]
struct NetworkInfoResponse {
    data: NetworkInfoData,
}

#[derive(Deserialize)]
struct NetworkInfoData {
    asns: Vec<String>,
    prefix: Option<String>,
}

#[derive(Deserialize)]
struct AsOverviewResponse {
    data: AsOverviewData,
}

#[derive(Deserialize)]
struct AsOverviewData {
    holder: Option<String>,
}

#[derive(Deserialize)]
struct WhoisResponse {
    data: WhoisData,
}

#[derive(Deserialize)]
struct WhoisData {
    records: Vec<Vec<WhoisField>>,
    #[serde(default)]
    irr_records: Vec<Vec<WhoisField>>,
}

#[derive(Deserialize)]
struct WhoisField {
    key: String,
    value: String,
}

#[derive(Default)]
struct GroupBuilder {
    asn: Option<u32>,
    as_holder: Option<String>,
    prefix: Option<String>,
    netname: Option<String>,
    description: Option<String>,
    country: Option<String>,
    maintainer: Option<String>,
    ip_addrs: Vec<IpAddr>,
}

#[tracing::instrument(skip_all, fields(ip_count = ip_addrs.len()))]
pub async fn enrich(
    ip_addrs: &HashSet<IpAddr>,
    company_name: &str,
    domains: &HashSet<String>,
) -> Vec<InfrastructureGroup> {
    let client = reqwest::Client::new();
    let mut groups = BTreeMap::<String, GroupBuilder>::new();

    for &ip_addr in ip_addrs {
        let metadata = fetch_ip_metadata(&client, ip_addr)
            .await
            .unwrap_or_else(|error| {
                tracing::warn!(%ip_addr, ?error, "Failed to enrich IP address with RIPEstat");
                IpMetadata {
                    ip_addr,
                    asn: None,
                    as_holder: None,
                    prefix: None,
                    netname: None,
                    description: None,
                    country: None,
                    maintainer: None,
                }
            });

        let key = group_key(&metadata);
        let group = groups.entry(key).or_default();
        merge_metadata(group, metadata);
    }

    let own_tokens = ownership_tokens(company_name, domains);
    groups
        .into_values()
        .map(|mut group| {
            group.ip_addrs.sort_unstable();
            let (kind, reason) = classify_group(&group, &own_tokens);

            InfrastructureGroup {
                asn: group.asn,
                as_holder: group.as_holder,
                prefix: group.prefix,
                netname: group.netname,
                description: group.description,
                country: group.country,
                maintainer: group.maintainer,
                kind,
                reason,
                ip_addrs: group.ip_addrs,
            }
        })
        .collect()
}

async fn fetch_ip_metadata(
    client: &reqwest::Client,
    ip_addr: IpAddr,
) -> Result<IpMetadata, reqwest::Error> {
    let network_info = fetch_network_info(client, ip_addr).await?;
    let asn = network_info
        .data
        .asns
        .first()
        .and_then(|asn| asn.trim_start_matches("AS").parse::<u32>().ok());

    let as_holder = match asn {
        Some(asn) => fetch_as_holder(client, asn).await?,
        None => None,
    };
    let whois = fetch_whois(client, ip_addr).await?;

    Ok(IpMetadata {
        ip_addr,
        asn,
        as_holder,
        prefix: network_info
            .data
            .prefix
            .or_else(|| whois_value(&whois, "inetnum")),
        netname: whois_value(&whois, "netname"),
        description: whois_value(&whois, "descr"),
        country: whois_value(&whois, "country"),
        maintainer: whois_value(&whois, "mnt-by"),
    })
}

async fn fetch_network_info(
    client: &reqwest::Client,
    ip_addr: IpAddr,
) -> Result<NetworkInfoResponse, reqwest::Error> {
    let url = format!("{RIPESTAT_NETWORK_INFO_URL}?resource={ip_addr}");
    client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

async fn fetch_as_holder(
    client: &reqwest::Client,
    asn: u32,
) -> Result<Option<String>, reqwest::Error> {
    let url = format!("{RIPESTAT_AS_OVERVIEW_URL}?resource=AS{asn}");
    let response = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<AsOverviewResponse>()
        .await?;

    Ok(response.data.holder)
}

async fn fetch_whois(
    client: &reqwest::Client,
    ip_addr: IpAddr,
) -> Result<WhoisResponse, reqwest::Error> {
    let url = format!("{RIPESTAT_WHOIS_URL}?resource={ip_addr}");
    client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

fn whois_value(response: &WhoisResponse, key: &str) -> Option<String> {
    response
        .data
        .records
        .iter()
        .chain(response.data.irr_records.iter())
        .flat_map(|record| record.iter())
        .find(|field| field.key.eq_ignore_ascii_case(key) && !field.value.trim().is_empty())
        .map(|field| field.value.clone())
}

fn group_key(metadata: &IpMetadata) -> String {
    match (metadata.asn, metadata.prefix.as_deref()) {
        (Some(asn), Some(prefix)) => format!("as{asn}:{prefix}"),
        (Some(asn), None) => format!("as{asn}"),
        (None, Some(prefix)) => format!("prefix:{prefix}"),
        (None, None) => format!("ip:{}", metadata.ip_addr),
    }
}

fn merge_metadata(group: &mut GroupBuilder, metadata: IpMetadata) {
    group.asn = group.asn.or(metadata.asn);
    fill_missing(&mut group.as_holder, metadata.as_holder);
    fill_missing(&mut group.prefix, metadata.prefix);
    fill_missing(&mut group.netname, metadata.netname);
    fill_missing(&mut group.description, metadata.description);
    fill_missing(&mut group.country, metadata.country);
    fill_missing(&mut group.maintainer, metadata.maintainer);
    group.ip_addrs.push(metadata.ip_addr);
}

fn fill_missing(target: &mut Option<String>, value: Option<String>) {
    if target.is_none() && value.as_ref().is_some_and(|value| !value.trim().is_empty()) {
        *target = value;
    }
}

fn classify_group(
    group: &GroupBuilder,
    own_tokens: &HashSet<String>,
) -> (InfrastructureKind, String) {
    let evidence = evidence_text(group);
    if let Some(provider) = known_hosting_provider(&evidence) {
        return (
            InfrastructureKind::Hosting,
            format!("Known hosting/cloud provider: {provider}"),
        );
    }

    if let Some(token) = own_tokens
        .iter()
        .filter(|token| token.len() >= 4)
        .find(|token| evidence.contains(token.as_str()))
    {
        return (
            InfrastructureKind::Own,
            format!("Network metadata matches company/domain token: {token}"),
        );
    }

    (
        InfrastructureKind::Unknown,
        "RIPEstat metadata does not match company or known hosting providers".to_owned(),
    )
}

fn evidence_text(group: &GroupBuilder) -> String {
    [
        group.as_holder.as_deref(),
        group.netname.as_deref(),
        group.description.as_deref(),
        group.maintainer.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ")
    .to_lowercase()
}

fn known_hosting_provider(evidence: &str) -> Option<&'static str> {
    [
        "selectel",
        "timeweb",
        "reg.ru",
        "cloudflare",
        "yandex cloud",
        "vk cloud",
        "hetzner",
        "amazon",
        "google",
        "microsoft",
        "digitalocean",
        "beget",
        "sprinthost",
    ]
    .into_iter()
    .find(|provider| evidence.contains(provider))
}

fn ownership_tokens(company_name: &str, domains: &HashSet<String>) -> HashSet<String> {
    let mut tokens = text_tokens(company_name);
    for domain in domains {
        tokens.extend(domain_tokens(domain));
    }
    tokens
}

fn text_tokens(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|token| token.len() >= 3 && !is_common_token(token))
        .map(ToOwned::to_owned)
        .collect()
}

fn domain_tokens(domain: &str) -> HashSet<String> {
    domain
        .to_lowercase()
        .split(['.', '-', '_', '/'])
        .filter(|token| token.len() >= 3 && !is_common_token(token))
        .map(ToOwned::to_owned)
        .collect()
}

fn is_common_token(token: &str) -> bool {
    matches!(
        token,
        "www"
            | "com"
            | "net"
            | "org"
            | "ru"
            | "рф"
            | "ooo"
            | "ооо"
            | "zao"
            | "пао"
            | "jsc"
            | "llc"
            | "ltd"
            | "inc"
            | "company"
    )
}
