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

        mod deserialize {
            use super::*;
            use serde_json::json;

            #[test]
            fn should_deseralise_string_as_string() {
                let one = "../one";
                let contents = json!(one).to_string();
                let result = serde_json::from_str::<Bin>(&contents).unwrap();
                assert_eq!(result, Bin::Single(one.to_string()));
            }

            #[test]
            fn should_deseralise_record_as_hashmap() {
                let contents = json!({ "one": "uno", "two": "duo" }).to_string();
                let result = serde_json::from_str::<Bin>(&contents).unwrap();
                let expected = HashMap::from([
                    ("one".to_string(), "uno".to_string()),
                    ("two".to_string(), "duo".to_string()),
                ]);
                assert_eq!(result, Bin::Record(expected));
            }
        }

        mod bins {
            use super::*;
            use std::convert::TryFrom;

            #[test]
            fn should_create_hashmap_of_package_name_binary_pair() {
                let single = "bin".to_string();
                let bin = Bin::Single(single.clone());
                let name = "name".to_string();
                let package_json = PackageJson {
                    bin,
                    name: name.clone(),
                    version: Version::try_from("1.0.0".to_string()).unwrap(),
                };
                let result = package_json.bins();
                let expected = HashMap::from([(name, single)]);
                assert_eq!(result, expected);
            }

            #[test]
            fn should_create_hashmap_from_record() {
                let bins = HashMap::from([
                    ("one".to_string(), "uno".to_string()),
                    ("two".to_string(), "duo".to_string()),
                ]);
                let name = "name".to_string();
                let bin = Bin::Record(bins.clone());
                let package_json = PackageJson {
                    bin,
                    name: name.clone(),
                    version: Version::try_from("1.0.0".to_string()).unwrap(),
                };
                let result = package_json.bins();

                assert_eq!(result, bins);
            }
        }
    }
}
