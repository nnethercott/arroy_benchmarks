use arroy::{Database, Reader, distances::Cosine};
use arroy_benchmarks::custom_ordered_float::NonNegativeOrderedFloat;
use core::f32;
use criterion::{BatchSize, BenchmarkId, Criterion, SamplingMode, criterion_group, criterion_main};
use heed::EnvOpenOptions;
use ordered_float::OrderedFloat;
use rand::{distributions::Uniform, prelude::*, rngs::OsRng};
use std::{
    cell::Cell, cmp::Reverse, collections::BinaryHeap, hint::black_box, num::NonZeroUsize,
    time::Duration,
};

//  sed "2q;d" vectors.txt | awk '{for (i=2; i<=NF; i++) print $i}' | wc -l

const DEFAULT_MAP_SIZE: usize = 1024 * 1024 * 1024 * 200;

// some data
fn data_gen(n: usize) -> Vec<(OrderedFloat<f32>, u32)> {
    let uniform = Uniform::new(0.0f32, 1.0);
    (0..n)
        .map(|i| {
            let dist = uniform.sample(&mut OsRng);
            (OrderedFloat(dist), i as u32)
        })
        .collect()
}

// some reversed data !
#[allow(dead_code)]
fn data_gen_rev(n: usize) -> Vec<Reverse<(OrderedFloat<f32>, u32)>> {
    let uniform = Uniform::new(0.0f32, 1.0);
    (0..n)
        .map(|i| {
            let dist = uniform.sample(&mut OsRng);
            Reverse((OrderedFloat(dist), i as u32))
        })
        .collect()
}

/// bench different top k strategies
#[allow(dead_code)]
fn theoretical_top_k(c: &mut Criterion) {
    let mut group = c.benchmark_group("theoretical");

    for n in [100, 1000, 10000] {
        'inner: for k in vec![10, 100, 1000] {
            if k > n {
                break 'inner;
            }
            group.bench_function(
                BenchmarkId::new("heap", format!("(n:{},k:{})", n, k)),
                move |b| {
                    b.iter_batched(
                        || data_gen_rev(n),
                        |v| {
                            let mut min_heap = BinaryHeap::from(v);
                            let mut output = Vec::with_capacity(k);

                            while let Some(Reverse((OrderedFloat(_dist), item))) = min_heap.pop() {
                                if output.len() == k {
                                    break;
                                }
                                output.push((item, 0.0f32));
                            }
                        },
                        criterion::BatchSize::LargeInput,
                    );
                },
            );

            group.bench_function(
                BenchmarkId::new("median", format!("(n:{},k:{})", n, k)),
                move |b| {
                    b.iter_batched(
                        || data_gen(n),
                        |v| {
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
                },
            );
        }
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

/// bench time for building binary heap from f32 wrappers
#[allow(dead_code)]
fn race_ordered_floats(c: &mut Criterion) {
    let mut group = c.benchmark_group("ord f32 race");

    for n in vec![10, 100, 1000] {
        group.bench_function(BenchmarkId::new("OrderedFloat", n), |b| {
            b.iter_batched(
                || data_gen(n),
                |v| {
                    let _ = black_box(BinaryHeap::from(v));
                },
                BatchSize::LargeInput,
            );
        });

        group.bench_function(BenchmarkId::new("NonNegativeOrderedFloat", n), |b| {
            b.iter_batched(
                // kinda wasteful but hey
                || {
                    let uniform = Uniform::new(0.0f32, 1.0);
                    (0..n)
                        .map(|i| {
                            let dist = uniform.sample(&mut OsRng);
                            (NonNegativeOrderedFloat(dist), i as u32)
                        })
                        .collect::<Vec<(NonNegativeOrderedFloat, u32)>>()
                },
                |v| {
                    let _ = black_box(BinaryHeap::from(v));
                },
                BatchSize::LargeInput,
            );
        });
    }
}

/// bench proxy for real code perf
#[allow(dead_code)]
fn reader_by_item(c: &mut Criterion) -> arroy::Result<()> {
    let mut group = c.benchmark_group("arroy");
    group
        .warm_up_time(Duration::from_secs(10))
        .measurement_time(Duration::from_secs(90))
        .sample_size(200)
        .nresamples(200_000)
        .sampling_mode(SamplingMode::Flat);

    // setup; 10 trees, 2k vectors, 768 dimensions
    let mut env_builder = EnvOpenOptions::new();
    env_builder.map_size(DEFAULT_MAP_SIZE);
    let env = unsafe { env_builder.open("assets/import.ary/") }.unwrap();
    let rtxn = env.read_txn()?;
    let database: Database<Cosine> = env.open_database(&rtxn, None)?.unwrap();
    let reader = Reader::open(&rtxn, 0, database)?;

    let _n_items = reader.n_items() as u32;
    let _counter = Cell::new(0);

    for nns in vec![10, 100, 1000] {
        for o in vec![1]{
            group.bench_function(BenchmarkId::new("reader", format!("(oversampling: {}, nns: {})", o, nns)), |b| {
                b.iter_batched(
                    || {
                        // let item = counter.get();
                        // counter.set((item + 1) % n_items);
                        (reader.nns(nns), 100)
                    },
                    |(mut builder, item)| {
                        let _ = builder
                            .oversampling(NonZeroUsize::new(o).unwrap()) // 10trees x 100
                            .by_item(&rtxn, item);
                    },
                    criterion::BatchSize::SmallInput,
                );
            });
        }
    }

    Ok(())
}

criterion_group!(
    benches,
    // theoretical_top_k,
    race_ordered_floats,
    // reader_by_item,
);
criterion_main!(benches);
