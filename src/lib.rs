/// CLI to download a binary for a system
/// Tell it the name of you binary
/// Tell it where your binary lives
///
/// We'll figure out the triple for your system and pass all that
/// into the URL provided.
mod package_json;
mod url_context;
mod version;

use clap::Parser;
use derive_more::From;
use futures::future::join_all;
use log::{debug, info, Level};
use neon::prelude::*;
use once_cell::sync::OnceCell;
use package_json::PackageJson;
use std::{
    env::{args_os, current_dir},
    ffi::OsString,
    fs,
    io::{self, Cursor},
    path::PathBuf,
};
use target_lexicon::HOST;
use tokio::runtime::Runtime;
use url_context::BinaryUrlContext;

// todo - creating bin file maybe with text, creating empty bin files, not creating bin files
// --filter='glob/**', bin name or file path? probably bins
// --placeholder=true|"text-message"|
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

// todo - used cached binary if nothing has changed via `cached_path`
async fn fetch_binary(url: String, destination: String) {
    let bytes = reqwest::get(url).await.unwrap().bytes().await.unwrap();
    let mut cursor = Cursor::new(bytes);
    let mut file = fs::File::create(destination).unwrap();
    io::copy(&mut cursor, &mut file).unwrap();
}

fn is_developing_locally(path: PathBuf) -> Option<bool> {
    let parent = path.parent()?.to_str()?;
    Some(!parent.ends_with("node_modules"))
}

#[derive(Debug, From)]
pub enum Error {
    IO(io::Error),
    PackageJson(package_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn args() -> Vec<OsString> {
    let args = args_os();
    debug!("cli arguments original:\n{:?}", args);

    let args: Vec<OsString> = args.skip(1).collect();
    debug!("cli arguments skipped:\n{:?}", args);

    args
}

fn run(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let args = args();

    let runtime = runtime(&mut cx)?;
    let channel = cx.channel();

    let args = Args::parse_from(args);
    let cwd = current_dir().unwrap();

    let pattern = args.url_pattern;
    let package_json = PackageJson::from_dir(cwd.clone())
        .map_err(Error::PackageJson)
        .or_else(|error| cx.throw_error(error.to_string()))?;

    let is_developing_locally = is_developing_locally(cwd).unwrap();
    debug!("running as library author:\n{}", is_developing_locally);

    let bins = package_json.clone().bins();

    if let true = is_developing_locally {
        info!("Looks like you're developing locally. Skipping postinstall process.");
    }

    let response = join_all(bins.into_iter().map(|(bin, file_dir)| {
        let url_context = BinaryUrlContext {
            bin,
            name: package_json.name.clone(),
            triple: HOST.to_string(),
            version: package_json.version.clone(),
        };
        let url = url_context.subsitute(&pattern);
        fetch_binary(url, file_dir)
    }));

    let (deferred, promise) = cx.promise();

    runtime.spawn(async move {
        debug!("spawn runtime process");
        response.await;

        deferred.settle_with(&channel, move |mut cx| Ok(cx.undefined()))
    });

    Ok(promise)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    console_log::init_with_level(log::Level::Debug).unwrap();
    cx.export_function("run", run)
}
