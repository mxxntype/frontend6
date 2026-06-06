use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[must_use]
pub struct EgrResponse {
    pub items: Vec<EgrResponseItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[must_use]
pub struct EgrResponseItem {
    #[serde(rename = "ЮЛ")]
    pub legal_entity: EgrResponseLegalEntity,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[must_use]
pub struct EgrResponseLegalEntity {
    #[serde(rename = "ИНН")]
    pub tin: String,
    #[serde(rename = "НаимСокрЮЛ")]
    pub short_name: String,
    #[serde(rename = "Контакты")]
    pub contacts: EgrResponseContacts,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[must_use]
pub struct EgrResponseContacts {
    #[serde(rename = "Сайт")]
    pub domains: Vec<String>,
    #[serde(rename = "Телефон")]
    pub cellphones: Vec<String>,
    #[serde(rename = "e-mail")]
    pub emails: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::EgrResponse;

    use serde_json::Value;

    #[rstest::rstest]
    #[case("../../assets/ozon.json")]
    fn deserialize_assets(#[case] path: &'static str) {
        let json = std::fs::read_to_string(path).unwrap();
        let _: EgrResponse = serde_json::from_str(&json).unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        let _: EgrResponse = serde_json::from_value(value).unwrap();
    }
}
