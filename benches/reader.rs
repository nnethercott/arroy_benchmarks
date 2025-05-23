use arroy::{Database, ItemId, QueryBuilder, Reader, Writer, distances::Cosine};
use arroy_benchmarks::custom_ordered_float::NonNegativeOrderedFloat;
use core::f32;
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use heed::{Env, EnvOpenOptions, RoTxn, WithTls};
use ordered_float::OrderedFloat;
use rand::{
    distributions::Uniform,
    prelude::*,
    rngs::{self, OsRng},
};
use std::{
    cell::Cell,
    cmp::Reverse,
    collections::BinaryHeap,
    fs,
    hint::black_box,
    num::NonZeroUsize,
    time::{Duration, Instant},
};

const N: usize = 100;
const DEFAULT_MAP_SIZE: usize = 1024 * 1024 * 1024 * 2;

//  sed "2q;d" vectors.txt | awk '{for (i=2; i<=NF; i++) print $i}' | wc -l
const DIMENSIONS: usize = 768;

// some helpers simulating the type of data in reader.nns(k).by_item()
fn data_gen() -> Vec<(OrderedFloat<f32>, u32)> {
    let uniform = Uniform::new(0.0f32, 1.0);

    (0..N)
        .map(|i| {
            let dist = uniform.sample(&mut OsRng);
            (OrderedFloat(dist), i as u32)
        })
        .collect()
}

// need Reverse for min binary heap
fn data_gen_rev() -> Vec<Reverse<(OrderedFloat<f32>, u32)>> {
    let uniform = Uniform::new(0.0f32, 1.0);

    (0..N)
        .map(|i| {
            let dist = uniform.sample(&mut OsRng);
            Reverse((OrderedFloat(dist), i as u32))
        })
        .collect()
}

/// bench time for building binary heap from f32 wrappers
fn race_ordered_floats(c: &mut Criterion) {
    let mut group = c.benchmark_group("ord f32 race");

    group.bench_function(BenchmarkId::from_parameter("Ordered"), |b| {
        b.iter_batched(
            data_gen,
            |v| {
                BinaryHeap::from(v);
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function(
        BenchmarkId::from_parameter("NonNegativeOrderedFloat"),
        |b| {
            b.iter_batched(
                || {
                    let uniform = Uniform::new(0.0f32, 1.0);

                    (0..N)
                        .map(|i| {
                            let dist = uniform.sample(&mut OsRng);
                            (NonNegativeOrderedFloat(dist), i as u32)
                        })
                        .collect::<Vec<(NonNegativeOrderedFloat, u32)>>()
                },
                |v| {
                    BinaryHeap::from(v);
                },
                BatchSize::LargeInput,
            );
        },
    );
}

/// bench different top k strategies
fn theoretical_top_k(c: &mut Criterion) {
    let mut group = c.benchmark_group("theoretical");

    for k in vec![10, 100, 1000] {
        group.bench_function(BenchmarkId::new("heap: O(n+k*log(n))", k), move |b| {
            b.iter_batched(
                data_gen_rev,
                |v| {
                    // Here we're inserting n items items into a binary heap
                    let mut min_heap = BinaryHeap::from(v);
                    let mut output = Vec::with_capacity(k);

                    while let Some(Reverse((OrderedFloat(dist), item))) = min_heap.pop() {
                        if output.len() == k {
                            break;
                        }
                        output.push((item, 0.0f32));
                    }
                },
                criterion::BatchSize::LargeInput,
            );
        });

        group.bench_function(BenchmarkId::new("median: O(n+k*log(k))", k), move |b| {
            b.iter_batched(
                data_gen,
                |mut v| {
                    let mut buffer = Vec::with_capacity(2 * k);
                    let mut v = v.into_iter();
                    buffer.extend((&mut v).take(k));
                    let mut threshold = OrderedFloat(f32::MAX);

                    for item in v {
                        if item.0 >= threshold {
                            continue;
                        }
                        if buffer.len() == 2 * k {
                            let (_, &mut median, _) = buffer.select_nth_unstable(k - 1);
                            threshold = median.0;
                            buffer.truncate(k);
                        }

                        // buffer resizing here?
                        // buffer.push(item);
                        let uninit = buffer.spare_capacity_mut();
                        uninit[0].write(item);
                        unsafe {
                            buffer.set_len(buffer.len() + 1);
                        }
                    }

                    // final sort
                    buffer.sort_unstable();
                    buffer.truncate(k);
                },
                criterion::BatchSize::LargeInput,
            );
        });

        // group.bench_function(BenchmarkId::new("O(n+k*log(n)*log(k))", k), move |b| {
        //     b.iter_batched(
        //         || data_gen().into_iter(),
        //         |mut v| {
        //             // fill binary heap with O(log(k)) insertions instead of dumping
        //             let mut heap = BinaryHeap::with_capacity(k);
        //             heap.extend((&mut v).take(k));
        //             let mut threshold = heap.peek().unwrap();
        //
        //             for item in v {
        //                 if item < *threshold {
        //                     let mut head = heap.peek_mut().unwrap(); // probably adds an overhead
        //                     *head = item;
        //                     drop(head);
        //                     threshold = heap.peek().unwrap();
        //                 }
        //             }
        //
        //             let mut output = Vec::with_capacity(k);
        //             while let Some((OrderedFloat(dist), item)) = heap.pop() {
        //                 if output.len() == k {
        //                     break;
        //                 }
        //                 output.push((item, 0.0f32));
        //             }
        //         },
        //         criterion::BatchSize::LargeInput,
        //     );
        // });
    }
}

/// bench proxy for real code perf
fn reader_by_item(c: &mut Criterion) -> arroy::Result<()> {
    let mut group = c.benchmark_group("arroy");
    // group.sample_size(20000);
    group.measurement_time(Duration::from_secs(30)).sample_size(1000);

    // setup
    let mut env_builder = EnvOpenOptions::new();
    env_builder.map_size(DEFAULT_MAP_SIZE);
    let env = unsafe { env_builder.open("assets/import.ary/") }.unwrap();
    let rtxn = env.read_txn()?;
    let database: Database<Cosine> = env.open_database(&rtxn, None)?.unwrap();
    let reader = Reader::open(&rtxn, 0, database)?;

    // deterministic sampling
    // let mut rng = StdRng::seed_from_u64(42);
    let n_items = reader.n_items() as u32;
    let counter = Cell::new(0);

    for nns in vec![10, 100, 1000] {
        group.bench_function(BenchmarkId::new("reader", nns), |b| {
            b.iter_batched(
                || {
                    let item = counter.get();
                    counter.set((item + 1) % n_items);
                    // let item: u32 = rng.gen_range(0..n_items);
                    (reader.nns(nns), item)
                },
                |(mut builder, index)| {
                    black_box(builder.by_item(&rtxn, index));
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    Ok(())
}

criterion_group!(
    benches,
    // race_ordered_floats,
    // theoretical_top_k,
    reader_by_item,
);
criterion_main!(benches);
