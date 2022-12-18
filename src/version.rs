use std::{convert::TryFrom, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone)]
#[serde(try_from = "String")]
pub struct Version(String);

impl TryFrom<String> for Version {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(version::Version::from_str(&value)?.to_string()))
    }
}
