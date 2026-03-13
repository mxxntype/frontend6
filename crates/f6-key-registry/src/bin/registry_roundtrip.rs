use std::str::FromStr;

use color_eyre::eyre::Report;
use f6_key_registry::fns::FnsApiKey;
use f6_key_registry::{ApiKey, ApiKeyRegistryStorage, KeyStatus};

fn main() -> Result<(), Report> {
    let _ = color_eyre::install();

    let fns_api_key_string = "d720bff6d7647a52f1db847e4760ee823af5e57d";
    let fns_api_key = FnsApiKey::from_str(fns_api_key_string)?;
    let api_key = ApiKey::from(fns_api_key);

    let (mut registry_storage, _temp_file) = ApiKeyRegistryStorage::create_temporary()?;

    registry_storage.registry.insert(api_key, KeyStatus::Usable);
    registry_storage.write()?;

    let storage_file_contents = std::fs::read_to_string(&registry_storage.file_path)?;
    eprintln!("contents of '{}':", registry_storage.file_path.display());
    eprintln!("{storage_file_contents}");

    Ok(())
}
