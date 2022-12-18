use std::{
    convert::{TryFrom, TryInto},
    env::current_dir,
    ffi::OsStr,
    fmt::Debug,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};
use url::Url;

use serde_json::json;
use tempdir::TempDir;

// tmp/to/package.json
// tmp/from/package.json

trait JsDirs
where
    Self: Into<PathBuf> + AsRef<OsStr>,
{
    /// Creates a `Url` with the file scheme `file:`
    fn url(&self) -> Option<Url> {
        let path_buf: PathBuf = self.into();
        let scheme = "file:";
        let url = scheme.to_string() + path_buf.to_str()?;
        let url = Url::parse(&url).unwrap();
        Some(url)
    }

    /// Append `"package.json"` to the end of `T`
    fn package_json(&self) -> PathBuf {
        let path_buf: PathBuf = self.into();
        path_buf.join("package.json")
    }
}

impl JsDirs for PathBuf {}

fn suck() {
    let temp_dir = TempDir::new("test").unwrap();
    let temp_dir = temp_dir.path();

    let js_dir = temp_dir.join("js");
    let rs_dir = temp_dir.join("rs");
    let pg_dir = current_dir().unwrap();
    let bs_dir = pg_dir.join("fixtures/binaries");

    // my js package where I want to install rust binaries
    let js_package_json = json!({
        "name": "my-js-package",
        "version": "9.1.0",
        "devDependencies": {
            "my-rust-package": rs_dir.url()
        }
    });

    let postinstall = format!("nirb {}/{{bin}}", bs_dir.to_str().unwrap());

    // my rust package that uses NIRB
    let rs_package_json = json!({
        "name": "my-rust-package",
        "version": "1.0.0",
        "bin": {
            "one": "./bin/one",
            "two": "./bin/two"
        },
        "scripts": {
            "postinstall": postinstall
        },
        "devDependencies": {
            "node-install-rust-binary": pg_dir.url()
        }
    });

    fs::create_dir_all(&js_dir).unwrap();
    fs::create_dir_all(&rs_dir).unwrap();

    fs::write(js_dir.package_json(), js_package_json.to_string()).unwrap();
    fs::write(rs_dir.package_json(), rs_package_json.to_string()).unwrap();

    // npm install
    let npm_install = Command::new("npm")
        .arg("install")
        .current_dir(js_dir)
        .status()
        .unwrap();

    if !npm_install.success() {
        panic!("npm install")
    }

    // npm exec {binary}
    let npm_exec = Command::new("npm").arg("exec").arg("one").status().unwrap();

    if !npm_exec.success() {
        panic!("npm_exec")
    }

    // npm version of `yarn exec`
}
