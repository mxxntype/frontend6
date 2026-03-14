use std::str::FromStr;

use color_eyre::eyre::Report;
use f6_api::Api;
use f6_api::fns::FnsApi;
use f6_key_registry::fns::FnsApiKey;
use f6_types::LegalEntityTIN;

const FNS_API_KEY_STRING: &str = "d720bff6d7647a52f1db847e4760ee823af5e57d";
const OZON_TIN: u64 = 7_704_217_370;

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    let fns_api_key = FnsApiKey::from_str(FNS_API_KEY_STRING)?;
    let ozon_tin = LegalEntityTIN::try_new(OZON_TIN)?;
    let response_body = FnsApi.fetch_egr(fns_api_key, &ozon_tin);

    println!("{response_body}");

    Ok(())
}
