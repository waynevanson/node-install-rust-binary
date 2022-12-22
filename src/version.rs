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
