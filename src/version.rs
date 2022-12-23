use std::{convert::TryFrom, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone)]
#[serde(try_from = "&str")]
pub struct Version(String);

impl TryFrom<&str> for Version {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(version::Version::from_str(&value)?.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // why does this decrease coverage? lol
    #[test]
    fn should_return_a_string_when_failing_to_apply_try_from_to_str() {
        let result = Version::try_from("bad-version").unwrap_err();
        let expected = "Invalid version format: expected 3 components, got 1.";
        assert_eq!(result, expected);
    }

    #[test]
    fn should_return_version() {
        let result = Version::try_from("1.0.0").unwrap();
        let expected = Version("1.0.0".to_string());
        assert_eq!(result, expected);
    }
}
