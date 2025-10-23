# PSLFS V2 - Test Plan

**Version:** 2.0
**Status:** Design Phase
**Last Updated:** 2025-10-22

---

## Table of Contents

1. [Testing Strategy](#testing-strategy)
2. [Unit Tests](#unit-tests)
3. [Integration Tests](#integration-tests)
4. [Property-Based Tests](#property-based-tests)
5. [Fuzz Testing](#fuzz-testing)
6. [Performance Tests](#performance-tests)
7. [FUSE Tests](#fuse-tests)
8. [Test Coverage Goals](#test-coverage-goals)

---

## Testing Strategy

### Testing Pyramid

```
           ╱╲
          ╱  ╲
         ╱ E2E ╲         5%  - End-to-end FUSE tests
        ╱──────╲
       ╱        ╲
      ╱Integration╲      20% - Integration tests
     ╱────────────╲
    ╱              ╲
   ╱  Unit Tests    ╲    75% - Unit tests
  ╱──────────────────╲
```

### Test Categories

1. **Unit Tests** - Individual functions and modules
2. **Integration Tests** - Multiple components working together
3. **Property-Based Tests** - Invariants and properties
4. **Fuzz Tests** - Random input validation
5. **Performance Tests** - Benchmarks and profiling
6. **FUSE Tests** - Real filesystem operations

### Tools

```toml
[dev-dependencies]
# Unit testing
criterion = "0.5"           # Benchmarking
tempfile = "3.8"            # Temporary files for testing

# Property testing
proptest = "1.4"            # Property-based testing
quickcheck = "1.0"          # Another property testing framework

# Fuzzing
cargo-fuzz = "0.11"         # Fuzz testing

# Coverage
tarpaulin = "0.27"          # Code coverage

# FUSE testing
assert_cmd = "2.0"          # CLI testing
assert_fs = "1.0"           # Filesystem assertions
```

---

## Unit Tests

### Module: `storage`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_file_backend_create() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.psl");

        let backend = FileBackend::create(&path, 1000).unwrap();
        assert_eq!(backend.total_blocks(), 1000);
    }

    #[test]
    fn test_file_backend_read_write() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.psl");
        let mut backend = FileBackend::create(&path, 10).unwrap();

        // Write block
        let data = Block::from_slice(b"Hello, World!");
        backend.write_block(BlockId(5), &data).unwrap();

        // Read block
        let read_data = backend.read_block(BlockId(5)).unwrap();
        assert_eq!(&read_data.data[..13], b"Hello, World!");
    }

    #[test]
    fn test_file_backend_bounds() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.psl");
        let backend = FileBackend::create(&path, 10).unwrap();

        // Should fail: out of bounds
        let result = backend.read_block(BlockId(100));
        assert!(matches!(result, Err(FsError::InvalidBlock(_))));
    }

    #[test]
    fn test_memory_backend() {
        let mut backend = MemoryBackend::new(10);

        let data = Block::from_slice(b"Test data");
        backend.write_block(BlockId(3), &data).unwrap();

        let read = backend.read_block(BlockId(3)).unwrap();
        assert_eq!(&read.data[..9], b"Test data");
    }
}
```

### Module: `allocator`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate() {
        let mut allocator = BlockAllocator::new(100);

        let block = allocator.allocate().unwrap();
        assert_eq!(block, BlockId(0));

        let block2 = allocator.allocate().unwrap();
        assert_eq!(block2, BlockId(1));
    }

    #[test]
    fn test_free() {
        let mut allocator = BlockAllocator::new(100);

        let block = allocator.allocate().unwrap();
        allocator.free(block).unwrap();

        // Should get same block again
        let block2 = allocator.allocate().unwrap();
        assert_eq!(block, block2);
    }

    #[test]
    fn test_allocate_all() {
        let mut allocator = BlockAllocator::new(10);

        // Allocate all blocks
        for i in 0..10 {
            let block = allocator.allocate().unwrap();
            assert_eq!(block, BlockId(i));
        }

        // Should fail: no more blocks
        assert!(allocator.allocate().is_none());
    }

    #[test]
    fn test_double_free() {
        let mut allocator = BlockAllocator::new(10);
        let block = allocator.allocate().unwrap();

        allocator.free(block).unwrap();

        // Should fail: already freed
        let result = allocator.free(block);
        assert!(matches!(result, Err(FsError::DoubleFree(_))));
    }

    #[test]
    fn test_is_allocated() {
        let mut allocator = BlockAllocator::new(10);

        assert!(!allocator.is_allocated(BlockId(0)));

        allocator.allocate().unwrap();
        assert!(allocator.is_allocated(BlockId(0)));
    }
}
```

### Module: `core::inode`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inode_create() {
        let inode = Inode::new(
            InodeId(1),
            InodeKind::File,
            0o644,
            1000,
            1000,
        );

        assert_eq!(inode.id, InodeId(1));
        assert_eq!(inode.kind, InodeKind::File);
        assert_eq!(inode.mode, 0o644);
        assert_eq!(inode.size, 0);
        assert_eq!(inode.nlink, 1);
    }

    #[test]
    fn test_inode_directory() {
        let inode = Inode::new(
            InodeId(2),
            InodeKind::Directory,
            0o755,
            1000,
            1000,
        );

        // Directories start with nlink=2 (. and ..)
        assert_eq!(inode.nlink, 2);
    }

    #[test]
    fn test_inode_blocks() {
        let mut inode = Inode::new(InodeId(1), InodeKind::File, 0o644, 0, 0);

        // Set blocks
        inode.set_block(0, BlockId(100)).unwrap();
        inode.set_block(1, BlockId(101)).unwrap();

        // Get blocks
        assert_eq!(inode.get_block(0), Some(BlockId(100)));
        assert_eq!(inode.get_block(1), Some(BlockId(101)));
        assert_eq!(inode.get_block(2), None);
    }

    #[test]
    fn test_inode_serialization() {
        let inode = Inode::new(InodeId(1), InodeKind::File, 0o644, 1000, 1000);

        // Serialize
        let bytes = bincode::serialize(&inode).unwrap();

        // Deserialize
        let deserialized: Inode = bincode::deserialize(&bytes).unwrap();

        assert_eq!(inode.id, deserialized.id);
        assert_eq!(inode.kind, deserialized.kind);
    }
}
```

### Module: `core::directory`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_new() {
        let dir = DirectoryData::new(InodeId(1), InodeId(2));

        // Should have . and .. entries
        assert!(dir.lookup(".").is_some());
        assert!(dir.lookup("..").is_some());
        assert_eq!(dir.entries.len(), 2);
    }

    #[test]
    fn test_directory_add() {
        let mut dir = DirectoryData::new(InodeId(0), InodeId(1));

        dir.add("file.txt".to_string(), InodeId(10), InodeKind::File).unwrap();

        let entry = dir.lookup("file.txt").unwrap();
        assert_eq!(entry.inode, InodeId(10));
        assert_eq!(entry.kind, InodeKind::File);
    }

    #[test]
    fn test_directory_duplicate() {
        let mut dir = DirectoryData::new(InodeId(0), InodeId(1));

        dir.add("file.txt".to_string(), InodeId(10), InodeKind::File).unwrap();

        // Should fail: already exists
        let result = dir.add("file.txt".to_string(), InodeId(11), InodeKind::File);
        assert!(matches!(result, Err(FsError::AlreadyExists(_))));
    }

    #[test]
    fn test_directory_remove() {
        let mut dir = DirectoryData::new(InodeId(0), InodeId(1));

        dir.add("file.txt".to_string(), InodeId(10), InodeKind::File).unwrap();

        let inode = dir.remove("file.txt").unwrap();
        assert_eq!(inode, InodeId(10));

        assert!(dir.lookup("file.txt").is_none());
    }

    #[test]
    fn test_directory_remove_nonexistent() {
        let mut dir = DirectoryData::new(InodeId(0), InodeId(1));

        let result = dir.remove("nonexistent.txt");
        assert!(matches!(result, Err(FsError::NotFound(_))));
    }
}
```

---

## Integration Tests

Located in `tests/` directory (separate from `src/`).

### Test: Create Filesystem

```rust
// tests/integration_test.rs
use pslfs::Filesystem;
use tempfile::tempdir;

#[test]
fn test_create_and_open_filesystem() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.psl");

    // Create filesystem
    {
        let fs = Filesystem::create(&path, 10 * 1024 * 1024, Some("TestFS")).unwrap();
        let stats = fs.stats();
        assert_eq!(stats.block_size, 4096);
        fs.close().unwrap();
    }

    // Reopen filesystem
    {
        let fs = Filesystem::open(&path, false).unwrap();
        let stats = fs.stats();
        assert_eq!(stats.block_size, 4096);
    }
}
```

### Test: File Operations

```rust
#[test]
fn test_file_create_read_write() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.psl");

    let mut fs = Filesystem::create(&path, 10 * 1024 * 1024, None).unwrap();
    let root = fs.root();

    // Create file
    let file = fs.create_file(root, "test.txt", 0o644).unwrap();

    // Write data
    let data = b"Hello, World!";
    let written = fs.write_file(file, 0, data).unwrap();
    assert_eq!(written, data.len());

    // Read back
    let mut buf = vec![0u8; 100];
    let read = fs.read_file(file, 0, &mut buf).unwrap();
    assert_eq!(read, data.len());
    assert_eq!(&buf[..read], data);
}
```

### Test: Directory Operations

```rust
#[test]
fn test_directory_operations() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.psl");

    let mut fs = Filesystem::create(&path, 10 * 1024 * 1024, None).unwrap();
    let root = fs.root();

    // Create directory
    let docs = fs.create_dir(root, "documents", 0o755).unwrap();

    // Create file in directory
    let file = fs.create_file(docs, "readme.txt", 0o644).unwrap();

    // Read directory
    let entries = fs.read_dir(root).unwrap();
    assert_eq!(entries.len(), 3); // ., .., documents

    // Lookup
    let found = fs.lookup(root, "documents").unwrap();
    assert_eq!(found, docs);
}
```

### Test: Path Resolution

```rust
#[test]
fn test_path_resolution() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.psl");

    let mut fs = Filesystem::create(&path, 10 * 1024 * 1024, None).unwrap();
    let root = fs.root();

    // Create hierarchy: /home/user/file.txt
    let home = fs.create_dir(root, "home", 0o755).unwrap();
    let user = fs.create_dir(home, "user", 0o755).unwrap();
    let file = fs.create_file(user, "file.txt", 0o644).unwrap();

    // Resolve path
    let resolved = fs.path_to_inode("/home/user/file.txt", None).unwrap();
    assert_eq!(resolved, file);
}
```

### Test: Permissions

```rust
#[test]
fn test_permissions() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.psl");

    let mut fs = Filesystem::create(&path, 10 * 1024 * 1024, None).unwrap();

    // Create users
    let alice = fs.create_user("alice", "pass123").unwrap();
    let bob = fs.create_user("bob", "pass456").unwrap();

    let root = fs.root();

    // Create file owned by alice, mode 0o600 (owner only)
    let file = fs.create_file(root, "private.txt", 0o600).unwrap();

    // Alice can read/write
    // Bob cannot read/write
    // (Need to add context/session to filesystem for this test)
}
```

---

## Property-Based Tests

Using `proptest` to test invariants.

### Property: Block allocation is consistent

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_allocate_free_consistent(
        ops in prop::collection::vec((prop::bool::ANY, 0u64..100), 1..100)
    ) {
        let mut allocator = BlockAllocator::new(100);
        let mut allocated = HashSet::new();

        for (alloc, block_id) in ops {
            if alloc {
                if let Some(block) = allocator.allocate() {
                    assert!(!allocated.contains(&block));
                    allocated.insert(block);
                }
            } else if !allocated.is_empty() {
                let block = BlockId(block_id % allocated.len() as u64);
                if allocated.remove(&block) {
                    allocator.free(block).unwrap();
                }
            }
        }

        // Invariant: allocated set matches allocator state
        for block_id in 0..100 {
            let block = BlockId(block_id);
            assert_eq!(
                allocated.contains(&block),
                allocator.is_allocated(block)
            );
        }
    }
}
```

### Property: Directory operations maintain invariants

```rust
proptest! {
    #[test]
    fn prop_directory_invariants(
        ops in prop::collection::vec(
            prop::string::string_regex("[a-z]{1,10}").unwrap(),
            1..20
        )
    ) {
        let mut dir = DirectoryData::new(InodeId(0), InodeId(1));
        let mut expected = HashMap::new();
        let mut next_inode = 100;

        for name in ops {
            // Try to add
            if !expected.contains_key(&name) {
                let inode = InodeId(next_inode);
                dir.add(name.clone(), inode, InodeKind::File).unwrap();
                expected.insert(name, inode);
                next_inode += 1;
            }
        }

        // Invariant: directory matches expected
        for (name, inode) in expected {
            let entry = dir.lookup(&name).unwrap();
            assert_eq!(entry.inode, inode);
        }
    }
}
```

---

## Fuzz Testing

Using `cargo-fuzz` to test crash safety.

### Fuzz Target: Superblock parsing

```rust
// fuzz/fuzz_targets/superblock.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use pslfs::Superblock;

fuzz_target!(|data: &[u8]| {
    // Try to deserialize random bytes
    if let Ok(sb) = bincode::deserialize::<Superblock>(data) {
        // If deserialization succeeds, checksum should verify
        // (or we should detect corruption)
        let _ = sb.verify_checksum();
    }
});
```

### Fuzz Target: Path parsing

```rust
// fuzz/fuzz_targets/path.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(path) = std::str::from_utf8(data) {
        // Should never panic on any path
        let _ = parse_path(path);
    }
});
```

---

## Performance Tests

Using `criterion` for benchmarking.

### Benchmark: Block allocation

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_allocate(c: &mut Criterion) {
    c.bench_function("allocate_1000", |b| {
        b.iter(|| {
            let mut allocator = BlockAllocator::new(10000);
            for _ in 0..1000 {
                black_box(allocator.allocate());
            }
        });
    });
}

criterion_group!(benches, bench_allocate);
criterion_main!(benches);
```

### Benchmark: File write

```rust
fn bench_file_write(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("bench.psl");

    c.bench_function("write_1kb", |b| {
        let mut fs = Filesystem::create(&path, 100 * 1024 * 1024, None).unwrap();
        let root = fs.root();
        let file = fs.create_file(root, "test.txt", 0o644).unwrap();

        let data = vec![0u8; 1024];

        b.iter(|| {
            black_box(fs.write_file(file, 0, &data).unwrap());
        });
    });
}
```

---

## FUSE Tests

End-to-end tests using real FUSE mount.

### Test: Mount and basic operations

```bash
#!/bin/bash
# tests/fuse_test.sh

set -e

# Create filesystem
./target/debug/pslfs init /tmp/test_fs --size 10M --user test --password test

# Mount
mkdir -p /tmp/test_mount
./target/debug/pslfs mount /tmp/test_fs /tmp/test_mount --user test --password test &
MOUNT_PID=$!

sleep 2

# Test operations
echo "Hello" > /tmp/test_mount/file.txt
cat /tmp/test_mount/file.txt | grep "Hello"

mkdir /tmp/test_mount/dir
ls /tmp/test_mount/dir

# Cleanup
fusermount3 -u /tmp/test_mount
kill $MOUNT_PID
rm -rf /tmp/test_fs /tmp/test_mount

echo "FUSE tests passed!"
```

### Test: Using real filesystem tools

```rust
use assert_cmd::Command;
use assert_fs::prelude::*;

#[test]
fn test_fuse_with_standard_tools() {
    let temp = assert_fs::TempDir::new().unwrap();
    let fs_path = temp.child("test.psl");
    let mount_path = temp.child("mount");

    // Create filesystem
    Command::cargo_bin("pslfs")
        .unwrap()
        .arg("init")
        .arg(fs_path.path())
        .arg("--size").arg("10M")
        .assert()
        .success();

    // Mount
    mount_path.create_dir_all().unwrap();
    let mut mount_cmd = Command::cargo_bin("pslfs")
        .unwrap()
        .arg("mount")
        .arg(fs_path.path())
        .arg(mount_path.path())
        .arg("-f")  // Foreground
        .spawn()
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(2));

    // Use standard Unix tools
    Command::new("touch")
        .arg(mount_path.child("test.txt").path())
        .assert()
        .success();

    Command::new("ls")
        .arg(mount_path.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("test.txt"));

    // Cleanup
    mount_cmd.kill().unwrap();
}
```

---

## Test Coverage Goals

### Coverage Targets

- **Overall:** >80% line coverage
- **Core modules:** >90% coverage
- **Error paths:** >70% coverage

### Running Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/index.html
```

---

## Continuous Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install FUSE
        run: sudo apt-get install -y fuse3 libfuse3-dev

      - name: Run tests
        run: cargo test --all-features

      - name: Run integration tests
        run: cargo test --test '*'

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Run clippy
        run: cargo clippy -- -D warnings

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Run coverage
        run: cargo tarpaulin --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3

  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run benchmarks
        run: cargo bench
```

---

## Test Execution Plan

### Phase 1: Unit Tests
```bash
# Run all unit tests
cargo test --lib

# Run specific module
cargo test --lib storage::tests

# With output
cargo test --lib -- --nocapture
```

### Phase 2: Integration Tests
```bash
# Run all integration tests
cargo test --test '*'

# Specific test
cargo test --test integration_test
```

### Phase 3: Property Tests
```bash
# Run property tests (slower)
cargo test prop_ -- --test-threads=1
```

### Phase 4: Fuzz Tests
```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run fuzzer
cargo fuzz run superblock

# With timeout
cargo fuzz run path -- -max_total_time=60
```

### Phase 5: Benchmarks
```bash
# Run all benchmarks
cargo bench

# Specific benchmark
cargo bench allocate

# Save baseline
cargo bench -- --save-baseline before
# Make changes...
cargo bench -- --baseline before
```

---

## Success Criteria

A test passes if:
- ✅ All assertions pass
- ✅ No panics occur
- ✅ No memory leaks (run with valgrind)
- ✅ No undefined behavior (run with miri)
- ✅ Performance within acceptable range

The entire test suite passes if:
- ✅ All unit tests pass
- ✅ All integration tests pass
- ✅ Property tests pass (10000+ iterations)
- ✅ Fuzz tests run without crashes (1 hour each)
- ✅ Code coverage >80%
- ✅ No clippy warnings
- ✅ Formatted with rustfmt

---

This comprehensive test plan ensures PSLFS V2 is:
- **Correct**: Does what it's supposed to do
- **Robust**: Handles edge cases and errors
- **Fast**: Meets performance targets
- **Safe**: No crashes or data corruption
- **Maintainable**: Tests document behavior
