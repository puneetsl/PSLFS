//! Performance benchmarks for the block allocator
//!
//! These benchmarks measure the performance of bitmap-based allocation
//! compared to the linked-list approach used in V0.1.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pslfs::allocator::BlockAllocator;
use pslfs::BlockId;

fn bench_allocate_single(c: &mut Criterion) {
    c.bench_function("allocate_single", |b| {
        b.iter(|| {
            let mut allocator = BlockAllocator::new(black_box(10000));
            black_box(allocator.allocate().unwrap());
        });
    });
}

fn bench_allocate_many(c: &mut Criterion) {
    c.bench_function("allocate_1000", |b| {
        b.iter(|| {
            let mut allocator = BlockAllocator::new(black_box(10000));
            for _ in 0..1000 {
                black_box(allocator.allocate().unwrap());
            }
        });
    });
}

fn bench_allocate_contiguous(c: &mut Criterion) {
    c.bench_function("allocate_contiguous_100", |b| {
        b.iter(|| {
            let mut allocator = BlockAllocator::new(black_box(10000));
            black_box(allocator.allocate_contiguous(black_box(100)).unwrap());
        });
    });
}

fn bench_free(c: &mut Criterion) {
    c.bench_function("free", |b| {
        b.iter(|| {
            let mut allocator = BlockAllocator::new(black_box(10000));
            let block = allocator.allocate().unwrap();
            black_box(allocator.free(black_box(block)).unwrap());
        });
    });
}

fn bench_is_allocated(c: &mut Criterion) {
    c.bench_function("is_allocated", |b| {
        b.iter(|| {
            let allocator = BlockAllocator::new(black_box(10000));
            black_box(allocator.is_allocated(black_box(BlockId(5000))));
        });
    });
}

fn bench_allocator_stats(c: &mut Criterion) {
    c.bench_function("stats", |b| {
        b.iter(|| {
            let mut allocator = BlockAllocator::new(black_box(10000));
            for _ in 0..1000 {
                allocator.allocate().unwrap();
            }
            black_box(allocator.stats());
        });
    });
}

fn bench_fragmented_allocation(c: &mut Criterion) {
    c.bench_function("fragmented_allocation", |b| {
        b.iter(|| {
            let mut allocator = BlockAllocator::new(black_box(1000));

            // Create fragmentation by allocating and freeing in pattern
            let mut blocks = Vec::new();
            for i in 0..100 {
                if i % 3 == 0 {
                    if let Some(block) = blocks.pop() {
                        allocator.free(block).unwrap();
                    }
                } else {
                    blocks.push(allocator.allocate().unwrap());
                }
            }

            // Now allocate in fragmented space
            for _ in 0..50 {
                black_box(allocator.allocate().unwrap());
            }
        });
    });
}

criterion_group!(
    benches,
    bench_allocate_single,
    bench_allocate_many,
    bench_allocate_contiguous,
    bench_free,
    bench_is_allocated,
    bench_allocator_stats,
    bench_fragmented_allocation
);
criterion_main!(benches);