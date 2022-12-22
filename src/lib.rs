/// CLI to download a binary for a system
/// Tell it the name of you binary
/// Tell it where your binary lives
///
/// We'll figure out the triple for your system and pass all that
/// into the URL provided.
mod package_json;
mod pairable;
mod url_context;
mod url_relative_file;
mod version;

use clap::Parser;
use derive_more::From;
use futures::{future::join_all, FutureExt};
use log::{debug, info};
use neon::prelude::*;
use once_cell::sync::OnceCell;
use package_json::PackageJson;
use std::{
    env::{args_os, current_dir},
    fs,
    io::{self, Cursor, Read},
    path::Path,
    str::FromStr,
};
use target_lexicon::HOST;
use tokio::runtime::Runtime;
use url::Url;
use url_context::BinaryUrlContext;

use crate::pairable::Pair;

// todo - creating bin file maybe with text, creating empty bin files, not creating bin files
// --filter=regex for bins --name --bin --other-property
// --placeholder=true|"text-message" (default false)
#[derive(Parser, Debug)]
#[command(author = "Wayne Van Son", version, about, long_about = None)]
struct Args {
    url_pattern: String,
}

/// Return a global tokio runtime or create one if it doesn't exist.
/// Throws a JavaScript exception if the `Runtime` fails to create.
fn runtime<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<&'static Runtime> {
    static RUNTIME: OnceCell<Runtime> = OnceCell::new();

    let runtime = RUNTIME
        .get_or_try_init(|| Runtime::new().or_else(|err| cx.throw_error(err.to_string())))?;

    debug!("tokio runtime initialised");

    Ok(runtime)
}

async fn fetch_binary(url: Url) -> Vec<u8> {
    if url.scheme() == "file" {
        fs::read(url.path()).unwrap()
    } else {
        reqwest::get(url)
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap()
            .bytes()
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap()
    }
}

// todo - used cached binary if nothing has changed via `cached_path`
async fn save_binary(bytes: Vec<u8>, destination: String) -> io::Result<()> {
    let mut cursor = Cursor::new(bytes);
    let mut file = fs::File::create(destination).unwrap();
    io::copy(&mut cursor, &mut file)?;
    Ok(())
}

fn is_developing_locally(path: &Path) -> Option<bool> {
    let parent = path.parent()?.to_str()?;
    Some(!parent.ends_with("node_modules"))
}

#[derive(Debug, From)]
pub enum Error {
    IO(io::Error),
    PackageJson(package_json::Error),
    Reqwest(reqwest::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// resolves a relative string to the url
fn url_relative(original: &str, cwd: &str) -> Option<String> {
    // "file"
    let scheme = &original[..4];

    if scheme != "file" {
        None
    } else {
        // "://" or ":", extract out ":" and compare if "//" exists
        // to determine next index
        // skip capturing 4 as it will be ":"
        let backslashes = &original[5..7];
        let path_index_start = if backslashes == "//" { 7 } else { 5 };

        let scheme_tail = &original[4..path_index_start];
        let path = &original[path_index_start..];

        let is_relative = path.starts_with("../") || path.starts_with("./");

        if is_relative {
            let path = scheme.to_string() + scheme_tail + cwd + "/" + path;
            Some(path)
        } else {
            None
        }
    }
}

fn run(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let args = args_os().skip(1);

    let args = Args::parse_from(args);
    let cwd = current_dir().unwrap();

    let pattern = args.url_pattern;
    let package_json = PackageJson::from_dir(&cwd)
        .map_err(Error::PackageJson)
        .or_else(|error| cx.throw_error(error.to_string()))?;

    let is_developing_locally = is_developing_locally(&cwd).unwrap();

    let bins = package_json.clone().bins();

    if let true = is_developing_locally {
        info!("Developing locally");
    }

    let response = join_all(
        bins.into_iter()
            .map(Pair::from)
            .map(|pair| {
                pair.map_first(|bin| {
                    let url_context = BinaryUrlContext {
                        bin,
                        name: package_json.name.clone(),
                        triple: HOST.to_string(),
                        version: package_json.version.clone(),
                    };

                    let url = url_context.subsitute(&pattern);
                    let url = url_relative(&url, cwd.to_str().unwrap()).unwrap_or(url);
                    let url = Url::from_str(&url).unwrap();

                    fetch_binary(url)
                })
            })
            .map(|pair| pair.into())
            .map(|(fut, file_dir)| fut.then(|bytes| save_binary(bytes, file_dir))),
    );

    let runtime = runtime(&mut cx)?;
    let channel = cx.channel();

    let (deferred, promise) = cx.promise();

    runtime.spawn(async move {
        debug!("spawn runtime process");
        let result = response.await;

        deferred.settle_with(&channel, move |mut cx| Ok(cx.undefined()))
    });

    Ok(promise)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    simple_logger::init().unwrap();
    cx.export_function("run", run)
}
