use crate::version::Version;
use derive_more::From;
use serde::{Deserialize, Serialize};
use tinytemplate::TinyTemplate;

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, From)]
pub struct UrlContext {
    pub bin: String,
    pub name: String,
    pub triple: String,
    pub version: Version,
}

// todo - make error type
impl UrlContext {
    pub fn subsitute(&self, url_pattern: &str) -> String {
        let mut tt = TinyTemplate::new();
        let name = "url";
        tt.add_template(name, url_pattern).unwrap();
        tt.render(name, self).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;
    use std::convert::TryFrom;

    #[test]
    fn should_serialise_to_json() {
        let name = "name".to_string();
        let bin = "bin".to_string();
        let version = Version::try_from("1.0.0").unwrap();
        let triple = "triple".to_string();

        let result = serde_json::to_string(&UrlContext {
            bin: bin.to_string(),
            name: name.to_string(),
            triple: triple.to_string(),
            version: version.clone(),
        })
        .unwrap();

        let expected = json!({
            "name": name,
            "version": version,
            "bin": bin,
            "triple": triple
        })
        .to_string();

        assert_eq!(result, expected);
    }

    #[test]
    fn should_subsitute_context_values() {
        let name = "name".to_string();
        let bin = "bin".to_string();
        let version = Version::try_from("1.0.0").unwrap();
        let triple = "triple".to_string();

        let pattern = "{name}{bin}{triple}-{version}";

        let result = UrlContext {
            bin: bin.to_string(),
            name: name.to_string(),
            triple: triple.to_string(),
            version: version.clone(),
        }
        .subsitute(pattern);

        let expected = "namebintriple-1.0.0";

        assert_eq!(result, expected);
    }
}
