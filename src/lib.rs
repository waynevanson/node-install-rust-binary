/// CLI to download a binary for a system
/// Tell it the name of you binary
/// Tell it where your binary lives
///
/// We'll figure out the triple for your system and pass all that
/// into the URL provided.
mod package_json;
mod pairable;
mod url_context;
mod version;

use clap::Parser;
use derive_more::From;
use futures::future::join_all;
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
use url_context::UrlContext;

use crate::{pairable::Pair, version::Version};

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

async fn fetch_binary(url: Url) -> Result<Vec<u8>> {
    if url.scheme() == "file" {
        fs::read(url.path()).map_err(Error::IO)
    } else {
        reqwest::get(url)
            .await
            .map_err(Error::Reqwest)?
            .bytes()
            .await
            .map_err(Error::Reqwest)?
            .bytes()
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Error::IO)
    }
}

// todo - used cached binary if nothing has changed via `cached_path`
fn save_binary(bytes: Vec<u8>, destination: String) -> Result<()> {
    let mut cursor = Cursor::new(bytes);
    let mut file = fs::File::create(destination)?;
    io::copy(&mut cursor, &mut file).map_err(Error::IO)?;
    Ok(())
}

async fn fetch_and_save_binary(url: Url, destination: String) -> Result<()> {
    let bytes = fetch_binary(url).await?;
    save_binary(bytes, destination)
}

fn is_developing_locally(path: &Path) -> Option<bool> {
    path.parent()
        .map(|parent| {
            parent
                .to_str()
                .map(|parent| !parent.starts_with("node_modules"))
        })
        .unwrap_or(Some(true))
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

    fn make_url(name: &str, version: &Version, bin: &str, pattern: &str, cwd: &str) -> Url {
        let url_context = UrlContext {
            bin: bin.to_string(),
            name: name.to_string(),
            triple: HOST.to_string(),
            version: version.clone(),
        };

        let url = url_context.subsitute(&pattern);
        let url = url_relative(&url, cwd).unwrap_or(url);
        let url = Url::from_str(&url).unwrap();
        url
    }

    let responses = join_all(
        bins.into_iter()
            .map(Pair::from)
            .map(|pair| {
                pair.map_first(|bin| {
                    make_url(
                        &package_json.name,
                        &package_json.version,
                        &bin,
                        &pattern,
                        cwd.to_str().unwrap(),
                    )
                })
            })
            .map(|pair| pair.into())
            .map(|(url, file_dir)| fetch_and_save_binary(url, file_dir)),
    );

    let runtime = runtime(&mut cx)?;
    let channel = cx.channel();

    let (deferred, promise) = cx.promise();

    runtime.spawn(async move {
        debug!("spawn runtime process");
        let errors = responses
            .await
            .into_iter()
            .filter_map(|result| result.err())
            .map(|error| error.to_string())
            .fold("".to_string(), |acc, curr| acc + ",\n" + &curr);

        deferred.settle_with(&channel, move |mut cx| {
            if errors.len() > 0 {
                let message = format!(
                    "Unable to resolve all binaries correctly:\n[\n{}\n]",
                    errors
                );
                cx.throw_error(message)
            } else {
                Ok(cx.undefined())
            }
        })
    });

    Ok(promise)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    simple_logger::init().unwrap();
    cx.export_function("run", run)
}
