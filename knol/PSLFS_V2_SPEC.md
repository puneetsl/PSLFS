# PSLFS V2 - Specification Document

**Version:** 2.0
**Status:** Design Phase
**Target Language:** Rust
**Author:** PSLFS Development Team
**Date:** 2025-10-22

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Goals and Non-Goals](#goals-and-non-goals)
3. [Design Principles](#design-principles)
4. [System Requirements](#system-requirements)
5. [Comparison with V0.1](#comparison-with-v01)
6. [References](#references)

---

## Executive Summary

PSLFS V2 is a modern rewrite of the original PSLFS (Puneet's Simple Learning File System) in Rust. It maintains the educational spirit and simplicity of V0.1 while introducing modern safety, performance, and usability improvements.

**Key Improvements:**
- Memory-safe implementation in Rust
- Improved performance with caching and larger block sizes
- Better error handling and recovery
- Cleaner API design
- Comprehensive test coverage
- Production-quality FUSE integration
- Backwards compatibility with V0.1 filesystem images (read-only initially)

**Educational Value Preserved:**
- Simple, understandable architecture
- Clear separation of concerns
- Well-commented code
- Extensive documentation
- Hackable and extensible design

---

## Goals and Non-Goals

### Primary Goals

1. **Educational Excellence**
   - Code that a college student can read and understand in a weekend
   - Clear documentation of every design decision
   - Example use cases and tutorials
   - Progressive complexity (simple core, advanced features optional)

2. **Memory Safety**
   - No segfaults, buffer overflows, or memory leaks
   - Leverage Rust's type system and ownership model
   - Safe concurrent access

3. **Correctness**
   - Crash-safe (no corruption on power loss during writes)
   - Strong consistency guarantees
   - Comprehensive error handling

4. **Modern Usability**
   - Mount as real filesystem via FUSE
   - Command-line tools with proper argument parsing
   - JSON/structured logging
   - Progress bars for long operations

5. **Performance (Reasonable)**
   - 10x faster than V0.1 (through caching and better algorithms)
   - Still significantly slower than ext4 (simplicity over speed)
   - Predictable performance characteristics

### Secondary Goals

- Cross-platform (Linux, macOS, Windows with winfsp)
- Optional compression support
- Simple snapshot/rollback mechanism
- Import/export to standard formats

### Non-Goals

1. **Not Production-Ready**
   - This is a learning tool, not a competitor to ext4/btrfs
   - No journaling or advanced recovery mechanisms
   - Simple encryption only (educational, not secure)

2. **Not High-Performance**
   - No complex B-trees or hash tables (keep it simple)
   - No complex caching algorithms
   - Acceptable to be 10-50x slower than production filesystems

3. **Not Feature-Complete**
   - No hard/symbolic links
   - No extended attributes (initially)
   - No quotas or ACLs
   - Basic permissions only

---

## Design Principles

### 1. Simplicity First
> "Simple is better than complex. Complex is better than complicated."

- Favor clarity over cleverness
- Use standard algorithms (even if not optimal)
- Avoid premature optimization
- Document the "why" not just the "what"

### 2. Safety Without Compromise
- Use Rust's type system to prevent bugs
- Make invalid states unrepresentable
- Panic on programmer errors, return errors for user errors
- No unsafe code in the core (FUSE bindings may use it)

### 3. Testability
- Every module has unit tests
- Integration tests for filesystem operations
- Fuzzing for binary format parsing
- Property-based tests for invariants

### 4. Observable Behavior
- Structured logging at every level
- Metrics for operations (counts, latencies)
- Debug mode with verbose output
- Tools to inspect filesystem internals

### 5. Incremental Learning
```
Layer 1 (Simple):     Basic block device + files
Layer 2 (Practical):  Directories + permissions
Layer 3 (Advanced):   FUSE + caching
Layer 4 (Expert):     Crash recovery + optimization
```

Students can understand Layer 1 on day one, and progressively explore deeper layers.

---

## System Requirements

### Minimum Requirements
- **OS:** Linux (primary), macOS, Windows 10+
- **Rust:** 1.70+ (stable)
- **Memory:** 64MB for the filesystem daemon
- **Disk:** Any storage device (file-backed initially)

### Dependencies
- **Core:** `serde`, `bincode`, `thiserror`, `log`
- **CLI:** `clap`, `indicatif`, `env_logger`
- **FUSE:** `fuser` (Linux/macOS), `winfsp` (Windows)
- **Testing:** `proptest`, `tempfile`, `criterion`

### FUSE Requirements
- **Linux:** `fuse3` (libfuse3-dev)
- **macOS:** `macfuse` or `osxfuse`
- **Windows:** `winfsp`

---

## Comparison with V0.1

| Aspect | V0.1 (C) | V2 (Rust) | Improvement |
|--------|----------|-----------|-------------|
| **Safety** | Manual memory mgmt | RAII + ownership | No leaks/segfaults |
| **Error Handling** | Return codes, nulls | Result<T, E> | Explicit, composable |
| **Block Size** | 16 bytes | 4096 bytes (configurable) | 256x less overhead |
| **File Structure** | Linked lists | B-tree indices | O(log n) vs O(n) |
| **Caching** | None | LRU cache | 10-50x speedup |
| **Concurrency** | Single-threaded | Thread-safe | Parallel operations |
| **Testing** | Manual | Automated suite | Continuous validation |
| **Documentation** | Comments | Rustdoc + guides | Searchable, linked |
| **FUSE Integration** | Basic wrapper | Native implementation | Better performance |
| **Code Size** | ~2000 LOC | ~3000 LOC (estimated) | More features, similar complexity |

### What We Keep from V0.1

✅ **Philosophy:**
- Educational focus
- Simple sector-based storage
- Human-readable operation names
- Permission system concept

✅ **Concepts:**
- Separate files for metadata vs content
- Free sector lists
- Hierarchical directory structure
- User authentication

### What We Improve

🔄 **Architecture:**
- Replace linked lists with indexed structures
- Add write-ahead logging for crash safety
- Separate concerns (storage / indexing / FUSE)
- Pluggable backend (file, block device, memory)

🔄 **Data Structures:**
- Larger blocks (4KB default)
- Bitmaps for free space (not linked lists)
- In-memory directory cache
- Efficient serialization with `serde`

🔄 **User Experience:**
- Modern CLI with `clap`
- Rich error messages
- Progress indicators
- JSON output option

---

## References

### Influential File Systems
1. **ext2** - Simple, well-documented, good learning resource
2. **FUSE examples** - passthrough_fh, hello, memfs
3. **Plan 9** - Clean design, simple protocols
4. **SQLite** - Best practices for crash-safe storage

### Rust Resources
1. [The Rust Programming Language](https://doc.rust-lang.org/book/)
2. [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
3. [fuser crate documentation](https://docs.rs/fuser/)
4. [Writing an OS in Rust](https://os.phil-opp.com/)

### Papers
1. "The Design and Implementation of a Log-Structured File System" (Rosenblum & Ousterhout)
2. "Soft Updates: A Solution to the Metadata Update Problem in File Systems" (McKusick & Ganger)

---

## Next Steps

See the following documents for detailed design:
1. **ARCHITECTURE.md** - System architecture and component design
2. **DATA_STRUCTURES.md** - Detailed data structure specifications
3. **API_DESIGN.md** - Public API and module interfaces
4. **TEST_PLAN.md** - Testing strategy and test cases
5. **MIGRATION_GUIDE.md** - How to migrate from V0.1 to V2
