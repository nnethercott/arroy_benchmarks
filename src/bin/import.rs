use arroy::distances::{BinaryQuantizedCosine, Cosine};
use arroy_benchmarks::utils::{BuildArgs, build};
use clap::Parser;

fn main() {
    let args = BuildArgs::parse();
    build::<Cosine>(args.clone()).expect(&format!("failed to build with args {:?}", args));
}
