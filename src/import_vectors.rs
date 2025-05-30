//! Download the associated file at https://www.notion.so/meilisearch/Movies-embeddings-1de3258859f54b799b7883882219d266

use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use arroy::{Database, Distance, Writer};
use clap::Parser;
use heed::{EnvFlags, EnvOpenOptions};
use rand::SeedableRng;
use rand::rngs::StdRng;

/// 1 GiB
const DEFAULT_MAP_SIZE: usize = 1024 * 1024 * 1024 * 1;

#[derive(Parser, Default, Clone, Debug)]
#[command(author, version, about, long_about = None,)]
pub struct BuildArgs {
    /// Sets a custom database path.
    #[arg(default_value = "assets/import.ary")]
    pub database: PathBuf,

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
    #[arg(long)]
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

pub fn build<D: Distance>(args: BuildArgs) -> Result<()> {
    env_logger::init();

    let BuildArgs {
        database,
        map_size,
        dimensions,
        write_map,
        no_append,
        n_trees,
        n_vecs,
        seed,
    } = args;

    let mut rng = StdRng::seed_from_u64(seed);
    let file = File::open("assets/vectors.txt").unwrap();
    let reader = BufReader::new(&file);

    // fresh start each time
    if fs::exists(&database)? {
        fs::remove_dir_all(&database)?;
    }
    let _ = fs::create_dir_all(&database);

    // Open the environment with the appropriate flags.
    let flags = if write_map {
        EnvFlags::WRITE_MAP
    } else {
        EnvFlags::empty()
    };
    let mut env_builder = EnvOpenOptions::new();
    env_builder.map_size(map_size);
    unsafe { env_builder.flags(flags) };
    let env = unsafe { env_builder.open(&database) }.unwrap();

    let mut wtxn = env.write_txn().unwrap();
    let database: Database<D> = env.create_database(&mut wtxn, None)?;
    let writer = Writer::<D>::new(database, 0, dimensions);

    // The file look like that
    // === BEGIN vectors ===
    // 0, [0.010056925, -0.0045358953, 0.009904552, 0.0046241777, ..., -0.050245073]
    // === END vectors ===

    let now = Instant::now();
    let mut insertion_time = Duration::default();
    let mut count = 0;
    for line in reader.lines().take(n_vecs.unwrap_or(10_000) + 1) {
        let line = line?;
        if line.starts_with("===") {
            continue;
        }

        let (id, vector) = line.split_once(',').expect(&line);
        let id: u32 = id.parse()?;
        let vector: Vec<_> = vector
            .trim_matches(|c: char| c.is_whitespace() || c == '[' || c == ']')
            .split(',')
            .map(|s| s.trim().parse::<f32>().unwrap())
            .collect();

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
    let now = Instant::now();
    let mut builder = writer.builder(&mut rng);
    if let Some(n_trees) = n_trees {
        builder.n_trees(n_trees);
    }
    builder.build(&mut wtxn)?;
    wtxn.commit().unwrap();
    println!("Took {:.2?} to build", now.elapsed());
    Ok(())
}
