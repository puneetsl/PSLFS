# PSLFS V2 - Design Documentation

**A Modern, Educational Filesystem in Rust**

---

## 🎯 Overview

PSLFS V2 is a complete redesign of the PSLFS (Puneet's Simple Learning File System) in Rust. It preserves the educational spirit of V0.1 while adding modern safety, performance, and usability improvements.

**Status:** Design Phase Complete ✅

---

## 📚 Documentation Index

This repository contains comprehensive design documentation for PSLFS V2:

### 1. **[PSLFS_V2_SPEC.md](PSLFS_V2_SPEC.md)** - Main Specification
   - **Start here!** Executive summary and project goals
   - Design principles and philosophy
   - Requirements and non-goals
   - Comparison with V0.1
   - References and further reading

### 2. **[ARCHITECTURE.md](ARCHITECTURE.md)** - System Architecture
   - Layered architecture design
   - Component breakdown
   - Data flow diagrams
   - Threading model
   - Error handling strategy
   - Module structure

### 3. **[DATA_STRUCTURES.md](DATA_STRUCTURES.md)** - Data Structures
   - On-disk format specification
   - In-memory structures
   - Serialization details
   - Disk layout
   - Size calculations
   - Compatibility notes

### 4. **[API_DESIGN.md](API_DESIGN.md)** - API Design
   - Public library API
   - CLI interface
   - FUSE integration
   - Error types
   - Usage examples
   - API stability guarantees

### 5. **[TEST_PLAN.md](TEST_PLAN.md)** - Testing Strategy
   - Unit tests
   - Integration tests
   - Property-based tests
   - Fuzz testing
   - Performance benchmarks
   - Coverage goals

### 6. **[MIGRATION_GUIDE.md](MIGRATION_GUIDE.md)** - Migration from V0.1
   - Implementation roadmap (12-week plan)
   - Data migration tools
   - Code translation guide
   - Learning path for Rust beginners
   - FAQ

### 7. **[BLOCK_DEVICE_GUIDE.md](BLOCK_DEVICE_GUIDE.md)** - Block Device Support
   - **Format USB drives and physical devices!**
   - BlockDeviceBackend implementation
   - CLI commands for formatting devices
   - Safety warnings and best practices
   - Usage examples (format pendrives, SD cards, etc.)
   - Performance characteristics

---

## 🚀 Quick Start

### For Implementers

If you're ready to start implementing PSLFS V2:

1. **Read the specs** (1-2 hours)
   - Read PSLFS_V2_SPEC.md for overview
   - Skim ARCHITECTURE.md for structure
   - Review DATA_STRUCTURES.md for details

2. **Set up environment** (30 minutes)
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Create project
   cargo new pslfs --lib
   cd pslfs

   # Add dependencies
   cargo add serde bincode thiserror log
   cargo add --dev tempfile proptest criterion
   ```

3. **Follow the roadmap** (12 weeks)
   - See MIGRATION_GUIDE.md for detailed schedule
   - Start with Phase 1: Foundation
   - Build incrementally with tests

### For Students

If you're learning about filesystems:

1. **Understand V0.1** (1 week)
   - Read CLAUDE.md for V0.1 overview
   - Run the existing PSLFS
   - Understand the basic concepts

2. **Learn Rust basics** (2-3 weeks)
   - [The Rust Book](https://doc.rust-lang.org/book/) chapters 1-10
   - [Rustlings](https://github.com/rust-lang/rustlings) exercises

3. **Study V2 design** (1 week)
   - Read all design documents
   - Compare with V0.1
   - Understand the improvements

4. **Implement incrementally** (8-12 weeks)
   - Follow the roadmap in MIGRATION_GUIDE.md
   - Build one module at a time
   - Write tests as you go

---

## 🎓 Learning Objectives

By implementing PSLFS V2, you will learn:

### Filesystem Concepts
- Block-based storage
- Inodes and directory structures
- Free space management (bitmaps)
- Path resolution
- File permissions and authentication
- FUSE integration

### Rust Programming
- Ownership and borrowing
- Type-driven design
- Error handling with Result/Option
- Binary serialization with serde
- Testing strategies
- Documentation with rustdoc

### Software Engineering
- Architecture design
- API design
- Test-driven development
- Performance optimization
- Documentation best practices
- Version control and CI/CD

---

## 📊 Key Improvements over V0.1

| Feature | V0.1 (C) | V2 (Rust) | Benefit |
|---------|----------|-----------|---------|
| **Safety** | Manual memory | Ownership system | No segfaults or leaks |
| **Performance** | ~40 KB/s | ~400 KB/s (est) | 10x faster |
| **Block Size** | 16 bytes | 4 KB | Less overhead |
| **Lookup** | O(n) linked list | O(log n) BTreeMap | Faster searches |
| **Caching** | None | LRU cache | Much faster repeated access |
| **Storage** | File-backed only | Files OR block devices | **Can format USB drives!** |
| **Testing** | Manual | Automated suite | Continuous validation |
| **Errors** | Printf debugging | Structured errors | Better diagnostics |
| **Documentation** | Comments | Rustdoc | Searchable, linked |

**Still 10-50x slower than ext4, but that's intentional - simplicity over speed!**

**New in V2:** You can actually format a USB drive or SD card with PSLFS! See [BLOCK_DEVICE_GUIDE.md](BLOCK_DEVICE_GUIDE.md) for details.

---

## 🏗️ Implementation Timeline

### Phase 1: Foundation (Weeks 1-2)
- ✅ Design complete
- ⏳ Storage backend
- ⏳ Block allocator
- ⏳ Basic tests

### Phase 2: Core Filesystem (Weeks 3-5)
- ⏳ Superblock
- ⏳ Inodes
- ⏳ Directories
- ⏳ File operations

### Phase 3: Performance (Week 6)
- ⏳ LRU cache
- ⏳ Benchmarks
- ⏳ Optimization

### Phase 4: CLI (Week 7)
- ⏳ Command parsing (clap)
- ⏳ Tools (init, ls, cat, etc)
- ⏳ User management

### Phase 5: FUSE (Weeks 8-9)
- ⏳ FUSE trait implementation
- ⏳ Mount/unmount
- ⏳ FUSE tests

### Phase 6: Migration (Week 10)
- ⏳ V0.1 format parser
- ⏳ Converter tool
- ⏳ Compatibility tests

### Phase 7: Polish (Weeks 11-12)
- ⏳ Documentation
- ⏳ Examples
- ⏳ Release preparation

**Total: ~12 weeks part-time, ~4-6 weeks full-time**

---

## 🛠️ Development Setup

### Prerequisites

```bash
# Rust (required)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# FUSE (for mounting)
# Ubuntu/Debian:
sudo apt-get install fuse3 libfuse3-dev pkg-config

# macOS:
brew install macfuse

# Arch Linux:
sudo pacman -S fuse3
```

### Project Structure

```
pslfs/
├── Cargo.toml           # Rust project manifest
├── src/
│   ├── lib.rs           # Library root
│   ├── main.rs          # CLI binary
│   ├── storage/         # Storage backends
│   ├── allocator/       # Block allocator
│   ├── core/            # Inodes, directories
│   ├── cache/           # LRU cache
│   ├── fs/              # Filesystem operations
│   ├── auth/            # User authentication
│   ├── fuse/            # FUSE integration
│   └── cli/             # CLI commands
├── tests/               # Integration tests
├── benches/             # Benchmarks
└── examples/            # Usage examples
```

---

## 📖 Design Principles

### 1. **Simplicity First**
> "Make it work, make it right, make it fast - in that order"

- Clear, readable code over clever tricks
- Standard algorithms (even if not optimal)
- Extensive documentation
- Progressive complexity

### 2. **Safety Without Compromise**
- Leverage Rust's type system
- Make invalid states unrepresentable
- Explicit error handling
- No unsafe code in core (FUSE bindings may use it)

### 3. **Educational Value**
- Code readable by college students
- Comments explain "why" not just "what"
- Examples and tutorials
- Hackable design

### 4. **Testability**
- Every module has unit tests
- Integration tests for workflows
- Property tests for invariants
- Fuzzing for robustness

---

## 🎯 Success Criteria

PSLFS V2 is successful if:

1. ✅ **It works**: Passes all tests, no data corruption
2. ✅ **It's safe**: No crashes, panics, or undefined behavior
3. ✅ **It's fast enough**: 10x faster than V0.1, usable for demos
4. ✅ **It's understandable**: College student can read and understand
5. ✅ **It's documented**: Every public API has examples
6. ✅ **It's testable**: >80% code coverage
7. ✅ **It's maintainable**: Clear structure, good error messages

---

## 🤝 Contributing

This is an educational project designed for learning. Contributions welcome!

### How to Contribute

1. **Pick a module** from the roadmap
2. **Read the design docs** for that module
3. **Implement with tests**
4. **Document your code**
5. **Submit a PR**

### Good First Issues

- [ ] Implement `MemoryBackend` storage
- [ ] Write unit tests for `Bitmap`
- [ ] Implement `pslfs ls` command
- [ ] Add examples to README
- [ ] Improve error messages

---

## 📝 License

Same as V0.1 - Educational use, feel free to learn from and build upon!

---

## 🙏 Acknowledgments

- **V0.1 PSLFS**: Foundation and inspiration
- **Rust Community**: Excellent tools and documentation
- **FUSE**: Making userspace filesystems possible
- **ext2**: Simple, well-documented design
- **Everyone learning systems programming**: This is for you!

---

## 📞 Support

- **Documentation**: Start with PSLFS_V2_SPEC.md
- **Questions**: Check MIGRATION_GUIDE.md FAQ
- **Issues**: File in GitHub issues
- **Learning**: See learning resources in MIGRATION_GUIDE.md

---

## 🗺️ Next Steps

### Ready to start? Here's your path:

1. **Today**: Read PSLFS_V2_SPEC.md (30 minutes)
2. **This week**: Set up Rust environment, read ARCHITECTURE.md
3. **Week 1-2**: Learn Rust basics if needed
4. **Week 3+**: Start implementing Phase 1

### Not ready to code?

- Study the design documents
- Compare with real filesystems (ext2, FUSE examples)
- Explore the Rust ecosystem
- Read the referenced papers

---

## 📚 Additional Resources

### Rust Learning
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings)

### Filesystem Design
- [FUSE Documentation](https://www.kernel.org/doc/html/latest/filesystems/fuse.html)
- [ext2 Specification](https://www.nongnu.org/ext2-doc/ext2.html)
- [OSTEP: File Systems](https://pages.cs.wisc.edu/~remzi/OSTEP/)

### Rust Filesystems
- [fuser crate](https://docs.rs/fuser/)
- [FUSE examples](https://github.com/cberner/fuser/tree/master/examples)

---

**Happy Hacking! 🚀**

*Remember: The goal isn't to build the fastest filesystem, it's to build understanding. Every line of code is a learning opportunity.*
