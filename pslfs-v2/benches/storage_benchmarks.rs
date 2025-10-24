//! Performance benchmarks for storage backends
//!
//! These benchmarks measure the performance of file-based and memory-based
//! storage operations, providing baseline metrics for optimization.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pslfs::storage::{FileBackend, MemoryBackend, StorageBackend};
use pslfs::{Block, BlockId};
use tempfile::tempdir;

fn bench_file_backend_create(c: &mut Criterion) {
    c.bench_function("file_backend_create", |b| {
        b.iter(|| {
            let dir = tempdir().unwrap();
            let path = dir.path().join("bench.psl");
            black_box(FileBackend::create(&path, black_box(1000)).unwrap());
        });
    });
}

fn bench_file_backend_read_write(c: &mut Criterion) {
    c.bench_function("file_backend_read_write", |b| {
        b.iter(|| {
            let dir = tempdir().unwrap();
            let path = dir.path().join("bench.psl");
            let mut backend = FileBackend::create(&path, black_box(1000)).unwrap();

            let data = Block::from_slice(black_box(b"Test data for benchmarking"));
            backend.write_block(black_box(BlockId(100)), &data).unwrap();
            black_box(backend.read_block(black_box(BlockId(100))).unwrap());
        });
    });
}

fn bench_file_backend_sequential_io(c: &mut Criterion) {
    c.bench_function("file_backend_sequential_io", |b| {
        b.iter(|| {
            let dir = tempdir().unwrap();
            let path = dir.path().join("bench.psl");
            let mut backend = FileBackend::create(&path, black_box(1000)).unwrap();

            let data = Block::from_slice(b"Sequential I/O test data");

            // Write sequentially
            for i in 0..100 {
                backend.write_block(BlockId(i), &data).unwrap();
            }

            // Read sequentially
            for i in 0..100 {
                black_box(backend.read_block(BlockId(i)).unwrap());
            }
        });
    });
}

fn bench_file_backend_random_io(c: &mut Criterion) {
    c.bench_function("file_backend_random_io", |b| {
        b.iter(|| {
            let dir = tempdir().unwrap();
            let path = dir.path().join("bench.psl");
            let mut backend = FileBackend::create(&path, black_box(1000)).unwrap();

            let data = Block::from_slice(b"Random I/O test data");
            let block_ids: Vec<BlockId> = (0..100)
                .map(|i| BlockId((i * 7) % 1000)) // Pseudo-random
                .collect();

            // Write randomly
            for block_id in &block_ids {
                backend.write_block(*block_id, &data).unwrap();
            }

            // Read randomly
            for block_id in &block_ids {
                black_box(backend.read_block(*block_id).unwrap());
            }
        });
    });
}

fn bench_memory_backend_read_write(c: &mut Criterion) {
    c.bench_function("memory_backend_read_write", |b| {
        b.iter(|| {
            let mut backend = MemoryBackend::new(black_box(1000));

            let data = Block::from_slice(black_box(b"Memory backend test data"));
            backend.write_block(black_box(BlockId(100)), &data).unwrap();
            black_box(backend.read_block(black_box(BlockId(100))).unwrap());
        });
    });
}

fn bench_memory_backend_bulk_operations(c: &mut Criterion) {
    c.bench_function("memory_backend_bulk_operations", |b| {
        b.iter(|| {
            let mut backend = MemoryBackend::new(black_box(1000));
            let data = Block::from_slice(b"Bulk operation test data");

            // Bulk write
            for i in 0..100 {
                backend.write_block(BlockId(i), &data).unwrap();
            }

            // Bulk read
            for i in 0..100 {
                black_box(backend.read_block(BlockId(i)).unwrap());
            }
        });
    });
}

fn bench_storage_sync(c: &mut Criterion) {
    c.bench_function("storage_sync", |b| {
        b.iter(|| {
            let dir = tempdir().unwrap();
            let path = dir.path().join("bench.psl");
            let mut backend = FileBackend::create(&path, black_box(1000)).unwrap();

            // Write some data
            let data = Block::from_slice(b"Sync test data");
            backend.write_block(BlockId(0), &data).unwrap();

            // Sync to disk
            black_box(backend.sync().unwrap());
        });
    });
}

criterion_group!(
    benches,
    bench_file_backend_create,
    bench_file_backend_read_write,
    bench_file_backend_sequential_io,
    bench_file_backend_random_io,
    bench_memory_backend_read_write,
    bench_memory_backend_bulk_operations,
    bench_storage_sync
);
criterion_main!(benches);