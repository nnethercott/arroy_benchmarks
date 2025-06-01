//! Download the associated file at https://www.notion.so/meilisearch/Movies-embeddings-1de3258859f54b799b7883882219d266
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, Instant};

use anyhow::Result;
use arroy::{Database, Distance, Writer};
use clap::Parser;
use heed::{EnvFlags, EnvOpenOptions};
use rand::{Rng, SeedableRng, rngs::StdRng};
use rand_distr::Normal;

/// 1 GiB
const DEFAULT_MAP_SIZE: usize = 1024 * 1024 * 1024 * 1;
const MOVIE_VECTORS: &'static str = "assets/vectors.txt";

#[derive(Parser, Default, Clone, Debug)]
#[command(author, version, about, long_about = None,)]
pub struct BuildArgs {
    /// dataset to use for benchmarks
    #[arg(long, default_value = "random")]
    pub dataset: TestDataset,

    /// Sets a custom database path.
    #[arg(default_value = "assets/import.ary")]
    pub temp_dir: PathBuf,

    /// Specify the size of the database.
    #[arg(long, default_value_t = DEFAULT_MAP_SIZE)]
    pub map_size: usize,

    /// The number of dimensions to construct the arroy tree.
    #[arg(long, default_value_t = 768)]
    pub dimensions: usize,

    /// Use the MDB_WRITEMAP option to reduce the memory usage of LMDB.
    #[arg(long)]
    pub write_map: bool,

    /// Do not try to append items into the database.
    #[arg(long, default_value_t = true)]
    pub no_append: bool,

    /// The number of tress to generate.
    #[arg(long)]
    pub n_trees: Option<usize>,

    /// The number of vectors to read in
    #[arg(long)]
    pub n_vecs: Option<usize>,

    /// The seed to generate the internal trees.
    #[arg(long, default_value_t = 42)]
    pub seed: u64,
}

#[derive(Default, Clone, Debug)]
pub enum TestDataset {
    Movies,
    #[default]
    Random, // TODO: make like Random(dim, mu, std) ...
}
impl FromStr for TestDataset {
    type Err = &'static str;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "movies" => Ok(TestDataset::Movies),
            "random" => Ok(TestDataset::Random),
            _ => Err("dataset does not exist"),
        }
    }
}

impl TestDataset {
    pub fn load(&self, n: usize) -> Vec<(u32, Vec<f32>)> {
        match &self {
            TestDataset::Movies => {
                // The file look like that
                // === BEGIN vectors ===
                // 0, [0.010056925, -0.0045358953, 0.009904552, 0.0046241777, ..., -0.050245073]
                // === END vectors ===
                let file = File::open(MOVIE_VECTORS).unwrap();
                let reader = BufReader::new(&file);

                let it = reader.lines().filter_map(|line| {
                    if line.is_ok() {
                        let line = line.unwrap();

                        if !line.starts_with("===") {
                            let (id, vector) = line.split_once(',').expect(&line);
                            let id: u32 = id.parse().ok()?;
                            let vector: Vec<f32> = vector
                                .trim_matches(|c: char| c.is_whitespace() || c == '[' || c == ']')
                                .split(',')
                                .map(|s| s.trim().parse::<f32>().unwrap())
                                .collect();

                            return Some((id, vector));
                        }
                    }
                    None
                });

                it.take(n).collect()
            }
            TestDataset::Random => {
                let mut rng = StdRng::seed_from_u64(42);
                let normal = Normal::new(10.0, 10.0).unwrap(); // mean=0, std_dev=1

                (0..n) // or however many items you need
                    .map(move |_| {
                        let key = rng.r#gen::<u32>();
                        let mut values: Vec<f32> = Vec::with_capacity(768);
                        for _ in 0..768 {
                            values.push(rng.sample(normal));
                        }
                        (key, values)
                    }).collect()
            }
        }
    }
}

pub fn build<D: Distance>(args: BuildArgs) -> Result<()> {
    env_logger::init();

    let BuildArgs {
        dataset,
        temp_dir,
        map_size,
        dimensions,
        write_map,
        no_append,
        n_trees,
        n_vecs,
        seed,
    } = args;

    // fresh start
    if no_append && fs::exists(&temp_dir)? {
        fs::remove_dir_all(&temp_dir)?;
    }
    let _ = fs::create_dir_all(&temp_dir);

    // 1. Open the environment with the appropriate flags
    let flags = if write_map {
        EnvFlags::WRITE_MAP
    } else {
        EnvFlags::empty()
    };
    let mut env_builder = EnvOpenOptions::new();
    env_builder.map_size(map_size);
    unsafe { env_builder.flags(flags) };
    let env = unsafe { env_builder.open(&temp_dir) }.unwrap();

    let mut wtxn = env.write_txn().unwrap();
    let database: Database<D> = env.create_database(&mut wtxn, None)?;
    let writer = Writer::<D>::new(database, 0, dimensions);

    // 2. insert items
    let now = Instant::now();
    let mut insertion_time = Duration::default();
    let mut count = 0;

    for (id, vector) in dataset.load(n_vecs.unwrap_or(10_000)) {
        let now = Instant::now();
        if no_append {
            writer.add_item(&mut wtxn, id, &vector)?;
        } else {
            writer.append_item(&mut wtxn, id, &vector)?;
        }
        insertion_time += now.elapsed();
        count += 1;
    }
    println!(
        "Took {:.2?} to parse and insert into arroy",
        now.elapsed() - insertion_time
    );
    println!("Took {insertion_time:.2?} insert into arroy");
    println!("There are {count} vectors");
    println!("Building the arroy internal trees...");

    // 3. build trees
    let now = Instant::now();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut builder = writer.builder(&mut rng);
    if let Some(n_trees) = n_trees {
        builder.n_trees(n_trees);
    }
    builder.build(&mut wtxn)?;
    wtxn.commit().unwrap();
    println!("Took {:.2?} to build", now.elapsed());
    Ok(())
}
