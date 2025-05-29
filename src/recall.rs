use anyhow::Result;
use arroy::distances::Euclidean;
use arroy_benchmarks::import_vectors::{Args, build};
use clap::Parser;
use heed::EnvOpenOptions;

const SAMPLE_DATA: &'static str = "assets/import.ary/";

fn run(build_args: Args) -> Result<()> {
    build::<Euclidean>(build_args)?;

    // now we're ready to query from it with a reader
    Ok(())
}

fn main() -> Result<()> {
    let build_args = Args::parse();
    run(build_args)?;

    Ok(())
}
