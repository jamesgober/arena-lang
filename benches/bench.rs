//! Benchmarks for the hot paths: allocating a node and resolving a handle.
//!
//! Allocation runs once per node in a parse, and resolution runs once per edge
//! every time a pass walks the tree, so both are measured here. The goal is to
//! confirm allocation stays a flat amortised cost as the arena grows and that
//! resolving a handle is a constant-time lookup.

use arena_lang::{Arena, Id};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_alloc(c: &mut Criterion) {
    let mut group = c.benchmark_group("alloc");

    for &count in &[1_024usize, 16_384, 262_144] {
        group.throughput(criterion::Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter(|| {
                // Reserve up front so the measurement is the bump, not growth.
                let mut arena = Arena::with_capacity(count);
                for i in 0..count {
                    black_box(arena.alloc(black_box(i)));
                }
                arena
            });
        });
    }

    group.finish();
}

fn bench_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("get");

    for &count in &[1_024usize, 16_384, 262_144] {
        let mut arena = Arena::with_capacity(count);
        let ids: Vec<Id<usize>> = (0..count).map(|i| arena.alloc(i)).collect();

        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, _| {
            b.iter(|| {
                // Resolve every handle once; sum to keep the loop observable.
                let mut acc = 0usize;
                for &id in &ids {
                    if let Some(v) = arena.get(black_box(id)) {
                        acc = acc.wrapping_add(*v);
                    }
                }
                black_box(acc)
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_alloc, bench_get);
criterion_main!(benches);
