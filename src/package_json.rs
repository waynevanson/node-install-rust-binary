use serde::Deserialize;
use std::{collections::HashMap, fs, io, path::PathBuf};

use crate::version::Version;

pub type FileName = String;
pub type FilePath = String;
pub type Bins = HashMap<FileName, FilePath>;

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(untagged)]
pub enum Bin {
    Single(String),
    Record(Bins),
}

#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct PackageJson {
    pub name: String,
    pub version: Version,
    pub bin: Bin,
}

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    SerdeJson(serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl PackageJson {
    pub fn from_dir(directory: PathBuf) -> Result<Self> {
        let path = directory.join("package.json");
        println!("{}", path.to_str().unwrap());
        let contents = fs::read(path).map_err(Error::IO)?;
        let result = serde_json::from_slice::<PackageJson>(&contents).map_err(Error::SerdeJson)?;
        Ok(result)
    }

    pub fn bins(self) -> Bins {
        match self.bin {
            Bin::Record(bins) => bins,
            Bin::Single(file_path) => HashMap::from([(self.name, file_path)]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod bin {
        use super::*;
        use serde_json::json;

        #[test]
        fn should_pass_valid_json() {
            let one = "../one";
            let contents = json!(one).to_string();
            let result = serde_json::from_str::<Bin>(&contents).unwrap();
            assert_eq!(result, Bin::Single(one.to_string()));
        }
    }
}
