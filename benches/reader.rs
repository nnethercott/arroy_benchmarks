use arroy::{Database, ItemId, QueryBuilder, Reader, Writer, distances::Cosine};
use core::f32;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use heed::{Env, EnvOpenOptions, RoTxn, WithTls};
use ordered_float::OrderedFloat;
use rand::{
    distributions::Uniform,
    prelude::*,
    rngs::{self, OsRng},
};
use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    fs,
    hint::black_box,
    num::NonZeroUsize,
    time::{Duration, Instant},
};

const N: usize = 1000;
const DEFAULT_MAP_SIZE: usize = 1024 * 1024 * 1024 * 2;

//  sed "2q;d" vectors.txt | awk '{for (i=2; i<=NF; i++) print $i}' | wc -l
const DIMENSIONS: usize = 768;

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

fn binary_heap_top_k(c: &mut Criterion) {
    let mut group = c.benchmark_group("min heap");

    for k in vec![10, 100, 1000] {
        group.bench_function(BenchmarkId::new("O(n+k*log(n))", k), move |b| {
            b.iter_batched(
                || data_gen_rev(),
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

fn median_top_k(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec");

    for k in vec![10, 100, 1000] {
        group.bench_function(BenchmarkId::new("O(n+k*log(k))", k), move |b| {
            b.iter_batched(
                || data_gen().into_iter(),
                |mut v| {
                    let mut buffer = Vec::with_capacity(2 * k);
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
    }
}

fn reader_by_item(c: &mut Criterion) -> arroy::Result<()> {
    let mut group = c.benchmark_group("actuel");
    group.warm_up_time(Duration::from_secs(5)).sample_size(1000);

    // setup
    let mut env_builder = EnvOpenOptions::new();
    env_builder.map_size(DEFAULT_MAP_SIZE);
    let env = unsafe { env_builder.open("assets/import.ary/") }.unwrap();
    let rtxn = env.read_txn()?;
    let database: Database<Cosine> = env.open_database(&rtxn, None)?.unwrap();
    let reader = Reader::open(&rtxn, 0, database)?;

    for nns in vec![10, 100, 1000] {
        group.bench_function(BenchmarkId::new("reader", nns), |b| {
            b.iter_batched(
                || {
                    let item: u32 = thread_rng().gen_range(0..10000);
                    (reader.nns(nns), item)
                },
                |(builder, index)| {
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
    // binary_heap_top_k,
    // median_top_k,
    reader_by_item,
);
criterion_main!(benches);
