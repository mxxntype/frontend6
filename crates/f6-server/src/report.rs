use std::collections::HashSet;
use std::fmt::Write;

use f6_types::LegalEntityTIN;
use f6_types::domain::DomainResponse;
use f6_types::fns::EgrResponseItem;
use f6_types::ip_addr::IpAddrResponse;
use f6_types::report::{InfrastructureKind, TINReport};
use tempfile::NamedTempFile;

#[tracing::instrument(skip_all, fields(%tin))]
pub async fn build(
    tin: LegalEntityTIN,
    egr_response: EgrResponseItem,
    DomainResponse(sublist3r_domains): DomainResponse,
    IpAddrResponse(ip_addrs): IpAddrResponse,
) -> TINReport {
    let legal_entity = egr_response.legal_entity;

    let known_domains = legal_entity
        .contacts
        .domains
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let domains = sublist3r_domains
        .union(&known_domains)
        .cloned()
        .collect::<HashSet<_>>();

    TINReport {
        tin,
        name: legal_entity.short_name,
        domains,
        ip_addrs,
        infrastructure_groups: Vec::new(),
    }
}

#[tracing::instrument(skip_all, fields(tin = %report.tin))]
pub fn build_pdf(report: &TINReport) {
    let TINReport {
        tin,
        name,
        domains,
        ip_addrs,
        infrastructure_groups,
    } = report;

    let mut typst_source = format!(
        "
= Отчёт о сканировании ИНН {tin}

Организация: *{name}*

== Обнаруженные домены и поддомены

",
    );

    let mut domains = domains.iter().collect::<Vec<_>>();
    domains.sort_unstable();
    for domain in domains {
        writeln!(typst_source, "- `{domain}`").unwrap();
    }

    writeln!(typst_source, "\n== Обнаруженные IP-адреса\n").unwrap();
    let mut ip_addrs = ip_addrs.iter().collect::<Vec<_>>();
    ip_addrs.sort_unstable();
    for ip_addr in ip_addrs {
        writeln!(typst_source, "- `{ip_addr}`").unwrap();
    }

    writeln!(typst_source, "\n== Группы инфраструктуры\n").unwrap();
    for group in infrastructure_groups {
        let title = match (group.asn, group.prefix.as_deref()) {
            (Some(asn), Some(prefix)) => format!("AS{asn} / {prefix}"),
            (Some(asn), None) => format!("AS{asn}"),
            (None, Some(prefix)) => prefix.to_owned(),
            (None, None) => "Сеть без ASN".to_owned(),
        };
        let kind = match group.kind {
            InfrastructureKind::Own => "собственная инфраструктура",
            InfrastructureKind::Hosting => "хостинг / облако",
            InfrastructureKind::Unknown => "неизвестно",
        };

        writeln!(typst_source, "=== {title}").unwrap();
        writeln!(typst_source, "Тип: *{kind}*").unwrap();
        write_optional(&mut typst_source, "Владелец AS", group.as_holder.as_deref());
        write_optional(&mut typst_source, "Netname", group.netname.as_deref());
        write_optional(&mut typst_source, "Описание", group.description.as_deref());
        write_optional(&mut typst_source, "Страна", group.country.as_deref());
        write_optional(&mut typst_source, "Maintainer", group.maintainer.as_deref());
        writeln!(typst_source, "Причина классификации: {}", group.reason).unwrap();
        writeln!(typst_source, "IP-адреса:").unwrap();

        for ip_addr in &group.ip_addrs {
            writeln!(typst_source, "- `{ip_addr}`").unwrap();
        }
        writeln!(typst_source).unwrap();
    }

    let temp_file = NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), typst_source).unwrap();

    let command_ok = std::process::Command::new("typst")
        .arg("compile")
        .arg(temp_file.path())
        .arg(format!("cache/report/{tin}.pdf"))
        .status()
        .unwrap()
        .success();
    assert!(command_ok, "typst failed!");
}

fn write_optional(typst_source: &mut String, label: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        writeln!(typst_source, "{label}: {value}").unwrap();
    }
}
