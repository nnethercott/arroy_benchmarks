use std::str::FromStr;

use anyhow::Result;
use arroy::{
    Database, Distance, Reader,
    distances::{Cosine, Euclidean, Manhattan},
};
use arroy_benchmarks::import_vectors::{BuildArgs, build};
use clap::Parser;
use fast_distances::{cosine, euclidean, manhattan};
use heed::EnvOpenOptions;
use ordered_float::OrderedFloat;
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use roaring::RoaringBitmap;

const SAMPLE_DATA: &'static str = "assets/import.ary/";

#[derive(Default, Clone)]
enum DistanceType {
    Cosine,
    #[default]
    Euclidean,
    Manhattan,
}
impl FromStr for DistanceType {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "cosine" => Ok(DistanceType::Cosine),
            "euclidean" => Ok(DistanceType::Euclidean),
            "manhattan" => Ok(DistanceType::Manhattan),
            _ => Err("not a valid distance"),
        }
    }
}

#[derive(Parser, Default)]
pub struct Args {
    /// args for building the index
    #[clap(flatten)]
    pub build_args: BuildArgs,

    #[clap(short, long, default_value = "Euclidean")]
    pub distance: DistanceType,

    #[clap(flatten)]
    pub run_args: RunArgs,
}

#[derive(Parser, Default, Clone)]
pub struct RunArgs{
    #[clap(short, long, default_value_t=None)]
    pub search_k: Option<usize>,
}

fn run<D: Distance + HowFar>(build_args: BuildArgs) -> Result<()> {
    build::<D>(build_args.clone())?;

    // build arroy::Reader
    let mut env_builder = EnvOpenOptions::new();
    env_builder.map_size(1024 * 1024 * 1024 * 1);
    let env = unsafe { env_builder.open(&build_args.database) }?;
    let rtxn = env.read_txn()?;
    let database: Database<D> = env.open_database(&rtxn, None)?.unwrap();
    let reader = Reader::open(&rtxn, 0, database)?;

    // get points
    let points: Vec<_> = reader.iter(&rtxn)?.map(|x| x.unwrap()).collect();

    // run sims
    let mut rng = StdRng::seed_from_u64(build_args.seed);
    let recalls: Vec<usize> = vec![1, 10, 20, 50, 100, 500];

    recalls.into_iter().map(|r| {
        let (_, query) = points.choose(&mut rng).unwrap();
        simulate(&reader, query, points.clone(), r, &build_args);
    });

    Ok(())
}

fn simulate<D: Distance + HowFar>(
    reader: &Reader<D>,
    query: &Vec<f32>,
    mut points: Vec<(u32, Vec<f32>)>,
    recall: usize,
    search_k: &BuildArgs,
) {
    // put these in order
    points.sort_unstable_by_key(|(_, p)| OrderedFloat(D::distance(query, p)));
    let points = &points[..recall];
    let mut relevant = RoaringBitmap::from_iter(points.iter().map(|(a, _)| *a));

    let retrieved = reader.nns(recall)
}

// todo: clean this
trait HowFar {
    fn distance(p: &[f32], q: &[f32]) -> f32;
}

impl HowFar for Euclidean {
    fn distance(p: &[f32], q: &[f32]) -> f32 {
        euclidean(&ndarray::aview1(p), &ndarray::aview1(q))
    }
}
impl HowFar for Cosine {
    fn distance(p: &[f32], q: &[f32]) -> f32 {
        cosine(&ndarray::aview1(p), &ndarray::aview1(q))
    }
}
impl HowFar for Manhattan {
    fn distance(p: &[f32], q: &[f32]) -> f32 {
        manhattan(&ndarray::aview1(p), &ndarray::aview1(q))
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.distance {
        DistanceType::Cosine => run::<Cosine>(args.build_args)?,
        DistanceType::Euclidean => run::<Euclidean>(args.build_args)?,
        DistanceType::Manhattan => run::<Manhattan>(args.build_args)?,
    }

    Ok(())
}
