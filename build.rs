use structopt::StructOpt;
use wasm_pack::command::build::{Build, BuildOptions, Target};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Build::try_from_opts(BuildOptions {
        path: Some("./pkg/world".into()),
        out_dir: "./../../static/wasm".into(),
        target: Target::Web,
        ..BuildOptions::from_args()
    })?
    .run()?;

    Ok(())
}
