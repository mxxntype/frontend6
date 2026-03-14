use std::str::FromStr;

use derive_more::Into;
use serde::{Deserialize, Serialize};

pub const KEY_LENGTH_CHARS: usize = 40;

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Into, Clone, Debug)]
pub struct FnsApiKey(String);

impl FromStr for FnsApiKey {
    type Err = FnsApiKeyParseError;

    fn from_str(key_string: &str) -> Result<Self, Self::Err> {
        let key_string_length = key_string.len();

        if key_string_length != KEY_LENGTH_CHARS {
            return Err(FnsApiKeyParseError::InvalidLength(key_string_length));
        }

        if let Some(bad_char) = key_string.chars().find(|char| !char.is_ascii_hexdigit()) {
            return Err(FnsApiKeyParseError::InvalidCharacter(bad_char));
        }

        Ok(Self(key_string.to_string()))
    }
}

impl std::fmt::Display for FnsApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FnsApiKeyParseError {
    #[error("The FNS key string must be {KEY_LENGTH_CHARS} chars long, but got {0}")]
    InvalidLength(usize),

    #[error("The FNS key string must be a hexadecimal string, character '{0}' is invalid")]
    InvalidCharacter(char),
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::FnsApiKey;

    #[rstest::rstest]
    #[case::actual_key("d720bff6d7647a52f1db847e4760ee823af5e57d")]
    fn keys_valid(#[case] key_string: &str) {
        let _api_key = FnsApiKey::from_str(key_string)
            .inspect(|api_key| eprintln!("{api_key:?}"))
            .unwrap();
    }

    #[rstest::rstest]
    #[case::invalid_chars("hioqBHhNBZbKSqy2OZYh9FcxUDxVcfVb7qlieMVY")]
    #[case::invalid_chars("8DNxq88x7Hx871NCgpESBSQgqey3aEj17Yc4kupd")]
    #[case::invalid_chars("ckgSXus6o5rxyisQWtCQJqvD2WfyE4SYrFLdZRys")]
    #[case::invalid_length("12345678")]
    #[case::invalid_length("abcdef12345678")]
    fn keys_invalid(#[case] key_string: &str) {
        let _err = FnsApiKey::from_str(key_string)
            .inspect_err(|error| eprintln!("{error:?}"))
            .unwrap_err();
    }
}
