use icann_rdap_client::prelude::*;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), RdapClientError> {
    let query = QueryType::from_str("185.73.194.82")?;

    let config = ClientConfig::default();
    let client = create_client(&config)?;
    let store = MemoryBootstrapStore::new();
    let response =
        rdap_bootstrapped_request(&query, &client, &store, |reg| eprintln!("fetching {reg:?}"))
            .await?;

    dbg!(&response.rdap);

    Ok(())
}
