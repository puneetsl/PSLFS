# PSLFS V0.1 to V2 - Migration Guide

**Version:** 2.0
**Status:** Design Phase
**Last Updated:** 2025-10-22

---

## Table of Contents

1. [Overview](#overview)
2. [What Changes](#what-changes)
3. [Implementation Roadmap](#implementation-roadmap)
4. [V0.1 Data Migration](#v01-data-migration)
5. [Code Translation Guide](#code-translation-guide)
6. [Learning Path](#learning-path)

---

## Overview

This guide helps you transition from PSLFS V0.1 (C) to V2 (Rust), whether you're:
- **Migrating data** from V0.1 filesystems to V2
- **Rewriting the codebase** from C to Rust
- **Learning** the new architecture

### Migration Strategy

We recommend a **phased approach**:

1. **Phase 1**: Read-only compatibility (V2 can read V0.1 filesystems)
2. **Phase 2**: Data converter tool (convert V0.1 → V2 format)
3. **Phase 3**: Full V2 implementation
4. **Phase 4**: Deprecate V0.1

---

## What Changes

### Conceptual Changes (Design Level)

| Aspect | V0.1 | V2 | Migration Strategy |
|--------|------|-----|-------------------|
| **Language** | C | Rust | Rewrite, using same concepts |
| **Block Size** | 16 bytes | 4096 bytes | Convert during migration |
| **Data Structures** | Linked lists | BTreeMap + Vec | Transform at load time |
| **Free Space** | Linked list | Bitmap | Rebuild during migration |
| **Storage** | 4 separate files | Same (compatible) | Keep same layout |
| **Caching** | None | LRU cache | Add new layer |
| **Error Handling** | Return codes | Result<T, E> | Explicit error types |
| **Concurrency** | Single-threaded | Thread-safe | Add locks/mutexes |

### What Stays the Same

✅ **Core concepts:**
- Sector-based storage
- Hierarchical directories
- User authentication
- File permissions
- FUSE integration

✅ **Educational value:**
- Still simple and understandable
- Clear documentation
- Hackable design

---

## Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)

**Goal**: Basic Rust project structure and core types

```
Week 1:
- [ ] Set up Rust project with Cargo
- [ ] Define core types (BlockId, InodeId, etc.)
- [ ] Implement Block and storage backend trait
- [ ] Write unit tests for storage backend
- [ ] Document design decisions

Week 2:
- [ ] Implement file-based storage backend
- [ ] Implement memory storage backend (for testing)
- [ ] Create bitmap allocator
- [ ] Unit tests for allocator
- [ ] Basic benchmarks
```

**Deliverable**: Working storage layer with tests

---

### Phase 2: Core Filesystem (Weeks 3-5)

**Goal**: Inode and directory operations

```
Week 3:
- [ ] Define Superblock structure
- [ ] Define Inode structure
- [ ] Implement inode serialization
- [ ] Create/read/write inodes
- [ ] Unit tests

Week 4:
- [ ] Define Directory structure
- [ ] Implement directory operations (add/remove/lookup)
- [ ] Path resolution logic
- [ ] Integration tests

Week 5:
- [ ] Implement file operations (create/read/write)
- [ ] Implement directory operations (mkdir/rmdir)
- [ ] Metadata operations (chmod/chown)
- [ ] Comprehensive integration tests
```

**Deliverable**: Working filesystem without FUSE

---

### Phase 3: Caching and Performance (Week 6)

**Goal**: Add caching layer

```
Week 6:
- [ ] Implement LRU cache
- [ ] Cache integration with filesystem
- [ ] Write-through logic
- [ ] Performance benchmarks
- [ ] Compare with V0.1 performance
```

**Deliverable**: Performant core filesystem

---

### Phase 4: CLI Tools (Week 7)

**Goal**: Command-line interface

```
Week 7:
- [ ] Implement `pslfs init`
- [ ] Implement `pslfs ls`
- [ ] Implement `pslfs cat`
- [ ] Implement `pslfs cp`
- [ ] Implement `pslfs check`
- [ ] CLI tests
```

**Deliverable**: Usable CLI tools

---

### Phase 5: FUSE Integration (Weeks 8-9)

**Goal**: Mount as real filesystem

```
Week 8:
- [ ] Implement FUSE trait
- [ ] Basic operations (lookup, getattr, read, write)
- [ ] Directory operations (readdir, mkdir, rmdir)
- [ ] Error conversions

Week 9:
- [ ] File operations (create, unlink)
- [ ] Metadata operations (setattr)
- [ ] FUSE tests
- [ ] Performance tuning
```

**Deliverable**: Working FUSE mount

---

### Phase 6: V0.1 Compatibility (Week 10)

**Goal**: Read V0.1 filesystems

```
Week 10:
- [ ] Parse V0.1 format
- [ ] Read V0.1 linked lists
- [ ] Convert to V2 in-memory structures
- [ ] Read-only FUSE mount of V0.1 filesystems
- [ ] Migration tool (V0.1 → V2 converter)
```

**Deliverable**: V0.1 migration tool

---

### Phase 7: Polish and Documentation (Week 11-12)

```
Week 11:
- [ ] Code documentation (rustdoc)
- [ ] User documentation
- [ ] Examples and tutorials
- [ ] Performance analysis

Week 12:
- [ ] Final testing
- [ ] Code review
- [ ] Release preparation
- [ ] Create demo video/presentation
```

**Deliverable**: Production-ready V2.0

---

## V0.1 Data Migration

### Reading V0.1 Format

V0.1 uses four binary files:
- `test1.psl` - Main partition (folders and files)
- `test1.fol` - Free folder sectors
- `test1.fil` - Free file sectors
- `test1.fs` - Free content sectors

### Migration Tool Design

```rust
/// Convert V0.1 filesystem to V2 format
pub struct V01Migrator {
    v01_path: PathBuf,
    v2_path: PathBuf,
}

impl V01Migrator {
    pub fn new(v01_path: PathBuf, v2_path: PathBuf) -> Self {
        Self { v01_path, v2_path }
    }

    pub fn migrate(&mut self) -> Result<MigrationReport> {
        // 1. Read V0.1 superblock/header
        let v01_header = self.read_v01_header()?;

        // 2. Create V2 filesystem
        let mut v2_fs = Filesystem::create(
            &self.v2_path,
            self.calculate_v2_size(&v01_header)?,
            Some("Migrated from V0.1"),
        )?;

        // 3. Migrate users
        self.migrate_users(&v01_header, &mut v2_fs)?;

        // 4. Traverse V0.1 directory tree
        let root_sector = v01_header.root_sector;
        self.migrate_directory(root_sector, v2_fs.root(), &mut v2_fs)?;

        // 5. Verify migration
        let report = self.verify_migration(&v2_fs)?;

        Ok(report)
    }

    fn migrate_directory(
        &self,
        v01_sector: u64,
        v2_parent: InodeId,
        v2_fs: &mut Filesystem,
    ) -> Result<()> {
        // Read V0.1 folder structure
        let v01_folder = self.read_v01_folder(v01_sector)?;

        // Migrate files in this directory
        let mut file_sector = v01_folder.filesector;
        while file_sector != 0 {
            let v01_file = self.read_v01_file(file_sector)?;
            self.migrate_file(&v01_file, v2_parent, v2_fs)?;
            file_sector = v01_file.next;
        }

        // Migrate subdirectories (recursive)
        let mut child_sector = v01_folder.insector;
        while child_sector != 0 {
            let v01_child = self.read_v01_folder(child_sector)?;

            // Create directory in V2
            let v2_child = v2_fs.create_dir(
                v2_parent,
                &v01_child.name,
                self.convert_permissions(&v01_child.properties),
            )?;

            // Recurse
            self.migrate_directory(child_sector, v2_child, v2_fs)?;

            child_sector = v01_child.next;
        }

        Ok(())
    }

    fn migrate_file(
        &self,
        v01_file: &V01File,
        v2_parent: InodeId,
        v2_fs: &mut Filesystem,
    ) -> Result<()> {
        // Create file in V2
        let v2_file = v2_fs.create_file(
            v2_parent,
            &v01_file.name,
            self.convert_permissions(&v01_file.properties),
        )?;

        // Read V0.1 file content (16-byte blocks linked list)
        let content = self.read_v01_file_content(v01_file.fsector)?;

        // Write to V2 file (4KB blocks)
        v2_fs.write_file(v2_file, 0, &content)?;

        Ok(())
    }

    fn read_v01_file_content(&self, mut fsector: u64) -> Result<Vec<u8>> {
        let mut content = Vec::new();

        // Follow linked list of 16-byte blocks
        while fsector != 0 {
            let block = self.read_v01_content_block(fsector)?;
            content.extend_from_slice(&block.data);
            fsector = block.next;
        }

        Ok(content)
    }
}
```

### Usage

```bash
# Using the migration tool
pslfs migrate /path/to/v01/test1 /path/to/v2/filesystem

# Output:
# Migrating PSLFS V0.1 to V2...
# Reading V0.1 filesystem...
# Creating V2 filesystem...
# Migrating 5 users...
# Migrating directory tree...
#   /: 10 files, 5 directories
#   /home: 3 subdirectories
#   /home/user: 25 files
# Total: 150 files, 32 directories
# Total size: 1.2 MB
# Migration complete!
# V2 filesystem: /path/to/v2/filesystem
```

---

## Code Translation Guide

### Example 1: Opening Filesystem

**V0.1 (C):**
```c
FILE *f = fopen("test1.psl", "r+");
if (!f) {
    printf("Error opening file\n");
    return;
}
fseek(f, sector, 0);
fread(&folder, sizeof(ffolder), 1, f);
fclose(f);
```

**V2 (Rust):**
```rust
let fs = Filesystem::open("/path/to/filesystem", false)?;
let inode = fs.metadata(inode_id)?;
// File automatically closed when fs goes out of scope
```

---

### Example 2: Creating a File

**V0.1 (C):**
```c
ffile file;
strcpy(file.name, filename);
file.next = 0;
file.prev = 0;
file.fsector = allocate_sector();
file.properties[0] = READ;
file.properties[1] = WRITE;

FILE *f = fopen("test1.psl", "r+");
fseek(f, file.sector, 0);
fwrite(&file, sizeof(ffile), 1, f);
fclose(f);
```

**V2 (Rust):**
```rust
let file_inode = fs.create_file(
    parent_inode,
    "filename",
    0o644,  // Read/write for owner, read for others
)?;
```

---

### Example 3: Reading a File

**V0.1 (C):**
```c
wfile block;
long sector = file.fsector;

while (sector != 0) {
    FILE *f = fopen("test1.fs", "r");
    fseek(f, sector, 0);
    fread(&block, sizeof(wfile), 1, f);
    fclose(f);

    printf("%s", block.buff);
    sector = block.next;
}
```

**V2 (Rust):**
```rust
let mut buffer = vec![0u8; 1024];
let bytes_read = fs.read_file(file_inode, 0, &mut buffer)?;
println!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));
```

---

### Example 4: Directory Traversal

**V0.1 (C):**
```c
ffolder folder;
readFolder(&folder, sector);

long child = folder.insector;
while (child != 0) {
    ffolder child_folder;
    readFolder(&child_folder, child);
    printf("%s\n", child_folder.name);
    child = child_folder.next;
}
```

**V2 (Rust):**
```rust
let entries = fs.read_dir(parent_inode)?;
for entry in entries {
    println!("{}", entry.name);
}
```

---

### Example 5: User Authentication

**V0.1 (C):**
```c
char username[20], password[20];
gets(username);
gets(password);

// Simple character shift encryption
char encrypted[20];
for (int i = 0; i < strlen(password); i++) {
    encrypted[i] = password[i] + username[0] + username[1] + username[2];
}

// Compare with stored password
if (strcmp(encrypted, stored_password) == 0) {
    printf("Authenticated!\n");
}
```

**V2 (Rust):**
```rust
use dialoguer::{Input, Password};

let username: String = Input::new()
    .with_prompt("Username")
    .interact()?;

let password: String = Password::new()
    .with_prompt("Password")
    .interact()?;

// Secure password hashing (bcrypt)
let user_id = fs.authenticate(&username, &password)?;
println!("Authenticated as user {:?}", user_id);
```

---

## Learning Path

### For Beginners (Never used Rust)

**Week 1-2: Learn Rust Basics**
1. Read [The Rust Book](https://doc.rust-lang.org/book/) chapters 1-10
2. Complete [Rustlings](https://github.com/rust-lang/rustlings) exercises
3. Understand: ownership, borrowing, lifetimes, Option/Result

**Week 3-4: Rust Ecosystem**
1. Learn Cargo (build system)
2. Understand modules and crates
3. Read documentation with rustdoc
4. Write tests

**Week 5-6: Systems Programming in Rust**
1. File I/O
2. Binary serialization (bincode, serde)
3. Error handling patterns
4. Concurrent programming basics

**Week 7+: Start PSLFS V2**
- Follow the implementation roadmap above
- Start with Phase 1 (Foundation)

---

### For C Programmers Switching to Rust

**Key Mindset Shifts:**

1. **Memory Management**
   - C: Manual malloc/free
   - Rust: Ownership system, RAII

2. **Error Handling**
   - C: Return codes, errno, NULL
   - Rust: Result<T, E>, Option<T>

3. **Strings**
   - C: null-terminated char*
   - Rust: String and &str (UTF-8)

4. **Arrays**
   - C: Raw pointers, manual bounds checking
   - Rust: Slices, automatic bounds checking

**Quick Reference:**

| C Pattern | Rust Equivalent |
|-----------|----------------|
| `malloc(size)` | `Vec::with_capacity(size)` or `Box::new()` |
| `free(ptr)` | Automatic when out of scope |
| `NULL` | `Option::None` |
| `if (!ptr)` | `if ptr.is_none()` |
| `return -1` (error) | `Err(FsError::...)` |
| `return 0` (success) | `Ok(())` |
| `FILE*` | `std::fs::File` |
| `fopen/fclose` | Automatic with RAII |
| `strcmp` | `==` operator |
| `strcpy` | `.clone()` or `.to_string()` |

---

### For PSLFS V0.1 Maintainers

**Recommended Approach:**

1. **Don't rewrite everything at once**
   - Start with one module (e.g., storage backend)
   - Get it working and tested
   - Move to next module

2. **Keep V0.1 running**
   - Use it as a reference
   - Compare outputs
   - Validate correctness

3. **Write tests first**
   - Define expected behavior
   - Test V0.1 behavior
   - Replicate in V2

4. **Document as you go**
   - Why decisions were made
   - What changed and why
   - Known issues and limitations

---

## Migration Checklist

### Before Starting

- [ ] Read all design documents
- [ ] Set up Rust development environment
- [ ] Complete Rust basics tutorial
- [ ] Understand PSLFS V0.1 architecture

### Phase 1: Foundation

- [ ] Create Cargo project
- [ ] Define core types
- [ ] Implement storage backend
- [ ] Write unit tests
- [ ] Document design decisions

### Phase 2: Core Filesystem

- [ ] Implement Superblock
- [ ] Implement Inode
- [ ] Implement Directory
- [ ] Implement file operations
- [ ] Integration tests

### Phase 3: Features

- [ ] Add caching
- [ ] Implement CLI
- [ ] FUSE integration
- [ ] User authentication

### Phase 4: Migration

- [ ] V0.1 format parser
- [ ] Migration tool
- [ ] Test with real V0.1 data
- [ ] Verify correctness

### Phase 5: Polish

- [ ] Documentation
- [ ] Examples
- [ ] Performance tuning
- [ ] Release!

---

## FAQ

**Q: Can I use V0.1 and V2 filesystems simultaneously?**
A: Yes! V2 can read V0.1 filesystems (read-only initially). Later, a converter tool will migrate data.

**Q: Will V2 be faster than V0.1?**
A: Yes, 10-50x faster due to larger blocks, caching, and better data structures. Still slower than production filesystems, but much more usable.

**Q: Do I need to know Rust?**
A: Basic Rust knowledge helps, but the code is designed to be educational. Start with Rust basics, then dive into the implementation.

**Q: How long will the migration take?**
A: For a college student working part-time: ~12 weeks to complete all phases. Full-time: ~4-6 weeks.

**Q: Can I contribute to V2 development?**
A: Absolutely! The project is designed for learning. Pick a module, implement it, write tests, submit a PR.

**Q: What if I get stuck?**
A: Check the documentation, examples, and tests. The design documents explain the "why" behind decisions. Start simple and iterate.

---

This migration guide provides a clear path from V0.1 to V2, whether you're migrating data, rewriting code, or learning Rust. Good luck! 🚀
