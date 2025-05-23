use arroy::{distances::Cosine, Database, Reader};
use heed::EnvOpenOptions;

const DEFAULT_MAP_SIZE: usize = 2*1024*1024;

fn main() -> arroy::Result<()>{
    let mut env_builder = EnvOpenOptions::new();
    env_builder.map_size(DEFAULT_MAP_SIZE);
    let env = unsafe { env_builder.open("assets/import.ary/") }.unwrap();
    let rtxn = env.read_txn()?;
    let database: Database<Cosine> = env.open_database(&rtxn, None)?.unwrap();
    let reader = Reader::open(&rtxn, 0, database)?;

    let builder = reader.nns(1000);

    for _ in 0..5000{
        let _ = builder.by_item(&rtxn, 42);
    }

    Ok(())
}
