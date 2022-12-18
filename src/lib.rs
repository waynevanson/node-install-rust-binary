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
use neon::prelude::*;
use once_cell::sync::OnceCell;
use package_json::PackageJson;
use std::{
    env::current_dir,
    fmt::Display,
    fs::{self},
    io::{self, Cursor},
    path::PathBuf,
};
use target_lexicon::HOST;
use tokio::runtime::Runtime;
use url_context::BinaryUrlContext;

#[derive(Parser, Debug)]
#[command(author = "Wayne Van Son", version, about, long_about = None)]
struct Args {
    url_pattern: String,
}

/// Return a global tokio runtime or create one if it doesn't exist.
/// Throws a JavaScript exception if the `Runtime` fails to create.
fn runtime<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<&'static Runtime> {
    static RUNTIME: OnceCell<Runtime> = OnceCell::new();

    RUNTIME.get_or_try_init(|| Runtime::new().or_else(|err| cx.throw_error(err.to_string())))
}

// todo - used cached binary if nothing has changed via `cached_path`
async fn fetch_binary(url: String, destination: String) {
    let bytes = reqwest::get(url).await.unwrap().bytes().await.unwrap();
    let mut cursor = Cursor::new(bytes);
    let mut file = fs::File::create(destination).unwrap();
    io::copy(&mut cursor, &mut file).unwrap();
}

fn is_developing_locally(path: PathBuf) -> Option<bool> {
    Some(!path.parent()?.to_str()?.ends_with("node_modules"))
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

async fn run_inner() -> Result<()> {
    let args = Args::parse();
    let cwd = current_dir()?;

    println!("{:?}", cwd);

    let pattern = args.url_pattern;
    let package_json = PackageJson::from_dir(cwd.clone())?;

    let is_developing_locally = is_developing_locally(cwd).unwrap();

    if is_developing_locally {
        println!("Looks like you're developing locally. Skipping postinstall process.");
    } else {
        let bins = package_json.clone().bins();

        join_all(bins.into_iter().map(|(bin, file_dir)| {
            let url_context = BinaryUrlContext {
                bin,
                name: package_json.name.clone(),
                triple: HOST.to_string(),
                version: package_json.version.clone(),
            };
            let url = url_context.subsitute(&pattern);
            fetch_binary(url, file_dir.to_string())
        }))
        .await;
    }

    Ok(())
}

fn run(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let rt = runtime(&mut cx)?;
    let channel = cx.channel();

    let (deferred, promise) = cx.promise();

    rt.spawn(async move {
        let ran = run_inner().await;

        deferred.settle_with(&channel, move |mut cx| {
            ran.or_else(|err| cx.throw_error(err.to_string()))?;

            Ok(cx.undefined())
        })
    });

    Ok(promise)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("run", run)
}
