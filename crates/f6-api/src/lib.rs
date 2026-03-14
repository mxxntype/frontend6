use f6_key_registry::fns::FnsApiKey;

pub trait Api {
    type ApiKeyType;

    fn fetch_egr(&self, api_key: Self::ApiKeyType, egr: &str) -> String;
}

pub struct FnsApi;

impl FnsApi {
    const FNS_API_EGR_URI: &str = "https://api-fns.ru/api/egr";
}

impl Api for FnsApi {
    type ApiKeyType = FnsApiKey;

    fn fetch_egr(&self, api_key: Self::ApiKeyType, egr: &str) -> String {
        let uri = format!(
            "{base_uri}?key={key}&req={egr}",
            base_uri = Self::FNS_API_EGR_URI,
            key = api_key,
        );

        ureq::get(uri)
            .call()
            .unwrap()
            .into_body()
            .read_to_string()
            .unwrap()
    }
}
