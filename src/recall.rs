use std::{num::NonZeroUsize, str::FromStr};

use anyhow::Result;
use arroy::{
    Database, Distance, Reader,
    distances::{Cosine, Euclidean, Manhattan},
};
use arroy_benchmarks::utils::{BuildArgs, build};
use clap::Parser;
use fast_distances::{cosine, euclidean, manhattan};
use heed::{EnvOpenOptions, RoTxn, WithTls};
use ordered_float::OrderedFloat;
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use rayon::prelude::*;
use rayon::slice::ParallelSliceMut;
use roaring::RoaringBitmap;

#[derive(Default, Clone, Debug)]
pub enum DistanceType {
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

#[derive(Parser, Default, Debug)]
pub struct Args {
    /// args for building the index
    #[clap(flatten)]
    pub build_args: BuildArgs,

    #[clap(short, long, default_value = "euclidean")]
    pub distance: DistanceType,

    #[clap(flatten)]
    pub run_args: RunArgs,
}

#[derive(Parser, Default, Clone, Debug)]
pub struct RunArgs {
    #[clap(short, long)]
    pub search_k: Option<usize>,

    #[clap(long)]
    pub n_samples: Option<u64>
}

fn run<D: Distance + HowFar>(args: Args) -> Result<()> {
    let build_args = args.build_args;
    let run_args = args.run_args;

    build::<D>(build_args.clone())?;

    let mut env_builder = EnvOpenOptions::new();
    env_builder.map_size(1024 * 1024 * 1024 * 1);
    let env = unsafe { env_builder.open(&build_args.temp_dir) }?;

    // build arroy::Reader
    let res = (0..run_args.n_samples.unwrap_or(10))
        .into_par_iter()
        .map(|i| {
            // create new rotxn
            let rtxn = env.read_txn()?;
            let database: Database<D> = env.open_database(&rtxn, None)?.unwrap();
            let reader = Reader::open(&rtxn, 0, database)?;

            // get points
            let points: Vec<_> = reader.iter(&rtxn)?.map(|x| x.unwrap()).collect();

            // simulate
            let mut rng = StdRng::seed_from_u64(build_args.seed + i);
            let recalls: Vec<usize> = vec![1, 10, 20, 50, 100, 500];
            let (_, query) = points.choose(&mut rng).unwrap();
            let scores =
                simulate(&reader, &rtxn, query, points.clone(), &recalls, &run_args).unwrap();

            // ok
            Ok::<Vec<f32>, anyhow::Error>(scores)
        })
        .collect::<Result<Vec<Vec<f32>>, anyhow::Error>>()?;

    println!("{:?}", reduce_mean_axis0(&res));
    Ok(())
}

fn simulate<D: Distance + HowFar>(
    reader: &Reader<D>,
    rtxn: &RoTxn<'_, WithTls>,
    query: &Vec<f32>,
    mut points: Vec<(u32, Vec<f32>)>,
    recalls: &[usize],
    run_args: &RunArgs,
) -> Result<Vec<f32>> {
    // put these in order
    points.par_sort_unstable_by_key(|(_, p)| OrderedFloat(D::distance(query, p)));

    let mut nns = reader.nns(recalls[recalls.len()-1]);
    if let Some(&search_k) = run_args.search_k.as_ref() {
        nns.search_k(NonZeroUsize::new(search_k).unwrap());
    }

    // /!\ top k is already ordered by relevance
    let neighbors = nns.by_vector(rtxn, query)?;

    let scores: Vec<_> = recalls
        .iter()
        .map(|&r| {
            // get ids of first `r` points
            let relevant = &points[..r];
            let relevant_bitmap = RoaringBitmap::from_iter(relevant.iter().map(|(a, _)| *a));

            // retain top `r` neighbors
            let retrieved_bitmap = RoaringBitmap::from_iter(neighbors.iter().take(r).map(|(a, _)| *a));

            (relevant_bitmap.intersection_len(&retrieved_bitmap) as f32) / (r as f32)
        })
        .collect();

    Ok(scores)
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

fn reduce_mean_axis0(matrix: &[Vec<f32>]) -> Vec<f32> {
    if matrix.is_empty() {
        return vec![];
    }

    let num_rows = matrix.len();
    let num_cols = matrix[0].len();
    let mut sum = vec![0.0; num_cols];

    for row in matrix {
        for (i, &val) in row.iter().enumerate() {
            sum[i] += val;
        }
    }

    sum.iter_mut().for_each(|x| *x /= num_rows as f32);
    sum
}

fn main() -> Result<()> {
    let args = Args::parse();

    dbg!("{:?}", &args);

    match args.distance {
        DistanceType::Cosine => run::<Cosine>(args).unwrap(),
        DistanceType::Euclidean => run::<Euclidean>(args).unwrap(),
        DistanceType::Manhattan => run::<Manhattan>(args).unwrap(),
    }

    Ok(())
}
