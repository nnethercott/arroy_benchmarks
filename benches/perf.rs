use criterion::{criterion_group, criterion_main, Criterion};
use rand::{Rng, thread_rng};
use std::{
    hint::black_box,
    mem::{self, MaybeUninit},
};

fn data(size: usize) -> Vec<u8> {
    let mut v = vec![0u8; size];
    thread_rng().fill(&mut v[..]);
    v
}

fn hot_loop_maybeuninit(data: &[u8]) -> Vec<u8> {
    let len = data.len();
    let mut out = vec![MaybeUninit::<u8>::uninit(); len];

    for (i, g) in data.into_iter().enumerate() {
        out[i] = MaybeUninit::new(*g);
    }

    unsafe { mem::transmute::<_, Vec<u8>>(out) }
}

fn hot_loop_with_capacity(data: &[u8]) -> Vec<u8> {
    let len = data.len();
    let mut out = Vec::with_capacity(len);

    for g in data.into_iter() {
        out.push(*g);
    }

    out
}

fn hot_loop_with_midground(data: &[u8]) -> Vec<u8> {
    let len = data.len();
    let mut out = Vec::with_capacity(len);

    for g in data.into_iter() {
        let uninit = out.spare_capacity_mut();
        uninit[0].write(*g);
        unsafe {
            out.set_len(out.len() + 1);
        }
    }

    out
}

fn maybeuninit(c: &mut Criterion) {
    let mut group = c.benchmark_group("vecs-and-stuff");

    group.bench_function("maybeuninit", |b| {
        b.iter_batched(
            || data(768),
            |v| {
                black_box(hot_loop_maybeuninit(&v));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("capacity", |b| {
        b.iter_batched(
            || data(768),
            |v| {
                black_box(hot_loop_with_capacity(&v));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("mid", |b| {
        b.iter_batched(
            || data(768),
            |v| {
                black_box(hot_loop_with_midground(&v));
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    maybeuninit,
);
criterion_main!(benches);
