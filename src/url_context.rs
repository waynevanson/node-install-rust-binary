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
