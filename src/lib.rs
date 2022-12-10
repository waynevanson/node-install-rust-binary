/// CLI to download a binary for a system
/// Tell it the name of you binary
/// Tell it where your binary lives
///
/// We'll figure out the triple for your system and pass all that
/// into the URL provided.
use std::env;

use clap::Parser;
use neon::prelude::*;
use target_lexicon::{Triple, HOST};
use url::Url;

use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

#[derive(Parser, Debug)]
#[command(author = "Wayne Van Son", version, about, long_about = None)]
struct Args {
    url: Url,
}

struct Provided {
    triple: Triple,
    base_url: Url,
}

impl Provided {
    fn url(&self) {}
}

impl Provided {
    fn new(base_url: Url) -> Self {
        Self {
            triple: HOST,
            base_url,
        }
    }
}

// Return a global tokio runtime or create one if it doesn't exist.
// Throws a JavaScript exception if the `Runtime` fails to create.
fn runtime<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<&'static Runtime> {
    static RUNTIME: OnceCell<Runtime> = OnceCell::new();

    RUNTIME.get_or_try_init(|| Runtime::new().or_else(|err| cx.throw_error(err.to_string())))
}

fn get_args() -> Args {
    let env_args = env::args_os().skip(1);
    let args = Args::parse_from(env_args);
    args
}

// save file to tmp dir, then copy to bin location
async fn get_binary_from_url(provided: Provided) {}

fn run(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let rt = runtime(&mut cx);
    let args = get_args();

    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("run", run)?;
    Ok(())
}
