use std::str::FromStr;

use color_eyre::eyre::Report;
use f6_api::{Api, FnsApi};
use f6_key_registry::fns::FnsApiKey;

const FNS_API_KEY_STRING: &str = "d720bff6d7647a52f1db847e4760ee823af5e57d";
const OZON_INN: &str = "7704217370";

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let fns_api_key = FnsApiKey::from_str(FNS_API_KEY_STRING)?;
    let response_body = FnsApi.fetch_egr(fns_api_key, OZON_INN);

    println!("{response_body}");

    Ok(())
}
