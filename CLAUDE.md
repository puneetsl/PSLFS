# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

PSLFS (Puneet's Simple Learning File System) is an educational virtual filesystem implementation. This repository contains:

1. **V0.1 (C Implementation)**: Working filesystem in C with FUSE support - located in `v0.1/`
2. **V2 (Rust Design)**: Complete design specifications for a modern Rust rewrite - located in `knol/`

### Repository Structure

```
PSLFS/
├── v0.1/              # Original C implementation
│   ├── basic.c        # Filesystem initialization
│   ├── fbase.c        # Main CLI interface
│   ├── pslfs_fuse*.c  # FUSE integration
│   └── *.h            # Header files
│
├── knol/              # V2 Design Documentation
│   ├── PSLFS_V2_README.md      # Start here! Overview and quick start
│   ├── PSLFS_V2_SPEC.md        # Main specification
│   ├── ARCHITECTURE.md         # System architecture
│   ├── DATA_STRUCTURES.md      # Data structures (also in v0.1/)
│   ├── API_DESIGN.md           # API design
│   ├── TEST_PLAN.md            # Testing strategy
│   └── MIGRATION_GUIDE.md      # V0.1 to V2 migration
│
└── CLAUDE.md          # This file
```

---

## V2 Design Documentation (knol/)

**Complete design specifications for PSLFS V2 - a modern Rust rewrite**

### Design Philosophy
- **Educational First**: Code readable by college students
- **Memory Safe**: Leverage Rust's ownership and type system
- **Performant**: 10x faster than V0.1 (but still educational, not production)
- **Well-Tested**: Comprehensive test suite with >80% coverage
- **Incremental Learning**: Layered architecture for progressive complexity

### Quick Start with V2 Design

1. **Read First**: `knol/PSLFS_V2_README.md` - Overview and documentation index
2. **Understand Goals**: `knol/PSLFS_V2_SPEC.md` - Project goals and principles
3. **Study Architecture**: `knol/ARCHITECTURE.md` - 7-layer architecture design
4. **Implementation Plan**: `knol/MIGRATION_GUIDE.md` - 12-week roadmap

### Key V2 Improvements

| Aspect | V0.1 (C) | V2 (Rust) |
|--------|----------|-----------|
| **Block Size** | 16 bytes | 4096 bytes |
| **Data Structures** | Linked lists | BTreeMap + Bitmap |
| **Safety** | Manual memory | Ownership system |
| **Performance** | ~40 KB/s | ~400 KB/s (target) |
| **Testing** | Manual | Automated suite |
| **Error Handling** | Return codes | Result<T, E> |

### V2 Implementation Status

- ✅ **Design Complete**: All 6 design documents finished
- ⏳ **Implementation**: Not started (waiting for implementer)
- 📅 **Timeline**: 12 weeks part-time, 4-6 weeks full-time

See `knol/MIGRATION_GUIDE.md` for the detailed implementation roadmap.

---

## V0.1 Implementation (v0.1/)

**Working C implementation with FUSE support**

The original PSLFS stores its data in binary files with custom sector-based allocation.

### Build and Run Commands

All V0.1 source files are in the `v0.1/` directory.

```bash
cd v0.1/

# Using Makefile (recommended)
make all          # Build both executables (base and fbase)
make clean        # Clean build artifacts and filesystem files
make install      # Build and initialize filesystem
make run          # Run fbase

# Manual compilation
gcc -Wall -o base basic.c
gcc -Wall -o fbase fbase.c

# FUSE filesystem wrapper (Linux only)
gcc -Wall -o pslfs_fuse pslfs_fuse_simple.c `pkg-config fuse3 --cflags --libs`
```

### Running the System

1. **Initialize the filesystem**: Run `base` first to create a new filesystem and register a user
   ```bash
   cd v0.1/
   ./base
   ```
   This creates four binary files: `test1.psl` (main partition), `test1.fol` (folder free sectors), `test1.fil` (file free sectors), `test1.fs` (file content sectors), plus `authent` (user credentials)

2. **Use the filesystem**: Run `fbase` to interact with the filesystem
   ```bash
   ./fbase
   ```
   Login with the username/password created during initialization

### Testing and Benchmarking

```bash
cd v0.1/

# Run system tests
./test_system.sh

# Run FUSE performance benchmarks (requires FUSE to be mounted)
./benchmark_pslfs.sh
```

### V0.1 Core Architecture

#### Binary File Storage Structure

The filesystem uses four binary files for storage:
- **`.psl` file**: Main partition containing all folder and file metadata structures
- **`.fol` file**: Free sector list for folders
- **`.fil` file**: Free sector list for files
- **`.fs` file**: Free sector list for file content blocks

#### Key Data Structures

**ffolder** (folder structure in `v0.1/fstest1.h`, `v0.1/fstest.h`, `v0.1/testio.h`):
- Linked list of folders with `next`/`prev` pointers for siblings
- `insector`: points to first child folder
- `filesector`: points to first file in this folder
- `upsector`: points to parent folder
- `properties[3]`: read/write/visibility permissions
- `sector`: location in `.psl` file

**ffile** (file structure):
- Linked list with `next`/`prev` pointers
- `fsector`: points to first content block in `.fs` file
- `properties[4]`: file permissions
- `sector`: location in `.psl` file

**wfile** (file content blocks):
- 16-byte chunks linked together
- Forms linked list for file contents using `next`/`prev`

**freesec** (free sector management):
- Linked list tracking available sectors in each binary file
- Used for allocation/deallocation

#### Memory Management

The system maintains three free sector lists in memory:
- `foldsec`: free sectors for folder metadata
- `filesec`: free sectors for file metadata
- `fsec`: free sectors for file content blocks

These are initialized by `invokeFreeSectors()` in `v0.1/fbase.c:32` and used throughout to allocate/deallocate space.

#### User Authentication

- User credentials stored in binary file header (`diskheader` in `v0.1/partition.h:2-12`)
- Password encryption using simple character shift based on first 3 chars of username (`v0.1/partition.h:79-102`)
- `Authenticate()` function validates user before filesystem access
- `makeUser()` creates initial user during filesystem setup

#### File/Folder Operations

All operations go through sector-based I/O:
- `readFolder()`: reads folder metadata from sector position
- `writeFolder()`: allocates sector and writes folder metadata
- `foldTravel()`: traverses folder linked list
- Similar patterns for file operations

Navigation uses sector pointers rather than paths internally, enabling efficient traversal.

### Command Line Interface

The main loop in `v0.1/fbase.c:33-273` implements a shell-like interface:

**Directory commands**:
- `ls` / `showdir`: list directory contents
- `cd <name>`: change directory (supports `/`, `..`, `-n`/`-p` for next/prev sibling)
- `md <name>`: create directory

**File commands**:
- `mf <name>`: create file (opens external editor)
- `show <name>`: display file contents
- `edit <name>`: edit file (choice of editors)
- `del <name>`: delete file or folder
- `df <name>`: delete file specifically

**Permission commands**:
- `cfmode <name>`: change file mode
- `cdmode <name>`: change directory mode

**Other**:
- `quit`: exit filesystem
- `clear`: clear screen
- `help`: show available commands
- `web <url>`: open URL in default browser

### Important V0.1 Implementation Notes

#### Multiple Header Versions

There are three versions of the filesystem core in `v0.1/`:
- `fstest.h`: original implementation
- `fstest1.h`: enhanced version with authentication and free sector management (used by fbase.c)
- `testio.h`: alternative implementation
- `conpsl.h`: contains basic data structure definitions and in-memory folder/file tree

The current system uses `fstest1.h` (included in `v0.1/basic.c:6` and `v0.1/fbase.c:8`).

#### Platform Compatibility

The code has been modernized for cross-platform support:
- Removed Windows-specific dependencies (`windows.h`, `ShellExecute`)
- `v0.1/ds.h` provides Unix terminal compatibility (`getch()`, `clrscr()`)
- Editor integration respects `$EDITOR` environment variable, falls back to `nano`/`vi`/`vim`
- URL opening uses `xdg-open` (Linux) or `open` (macOS)
- Replaced unsafe `gets()` with `fgets()` throughout
- Improved error handling with null checks and proper file closures

#### Hardcoded Paths

The default filesystem name is hardcoded in `v0.1/fbase.c:14-24` as `"test1"` with extensions `.psl`, `.fol`, `.fil`, `.fs`. To use different filesystem names, modify these path variables.

#### Sector-Based Architecture

The entire system uses file offsets as "sectors" - essentially treating binary files as a custom block device. All pointers between structures are file offsets, enabling direct fseek/fread/fwrite operations.

### V0.1 FUSE Filesystem Wrapper

PSLFS V0.1 can be mounted as a real Linux filesystem using FUSE (Filesystem in Userspace):

#### Setup and Mounting

```bash
cd v0.1/

# Install FUSE dependencies (Ubuntu/Debian)
sudo apt-get install fuse3 libfuse3-dev pkg-config

# Build the FUSE wrapper
gcc -Wall -o pslfs_fuse pslfs_fuse_simple.c `pkg-config fuse3 --cflags --libs`

# Create mount point and mount
mkdir /tmp/pslfs_mount
./pslfs_fuse /tmp/pslfs_mount

# Use like a normal filesystem
ls /tmp/pslfs_mount
cat /tmp/pslfs_mount/file.txt

# Unmount when done
fusermount3 -u /tmp/pslfs_mount
```

#### Implementation Files

- `v0.1/pslfs_fuse_simple.c`: Simplified FUSE implementation (recommended)
- `v0.1/pslfs_fuse.c`: Full-featured FUSE implementation
- See `v0.1/FUSE_MOUNT_GUIDE.md` for detailed performance analysis and optimization strategies

#### Performance Characteristics

The FUSE wrapper is significantly slower than native filesystems (20-100x slower than ext4) due to:
- No caching of metadata or file handles
- Linked list traversal (O(n) operations)
- Small 16-byte block size for file content
- High syscall overhead from repeated file open/close operations

Run `cd v0.1 && ./benchmark_pslfs.sh` for detailed performance benchmarks. Despite the slowdown, it's an excellent learning tool for understanding filesystem design tradeoffs.

---

## Working with This Repository

### For V2 Implementation

If you're implementing PSLFS V2 in Rust:
1. Start with `knol/PSLFS_V2_README.md`
2. Follow the roadmap in `knol/MIGRATION_GUIDE.md`
3. Reference V0.1 code in `v0.1/` as needed
4. Create new Rust project outside this repository (or in `pslfs-v2/` subdirectory)

### For V0.1 Development

If you're working on the C implementation:
1. All source files are in `v0.1/`
2. Build with `cd v0.1 && make all`
3. See `v0.1/README.md` for V0.1-specific documentation
4. FUSE guide: `v0.1/FUSE_MOUNT_GUIDE.md`

### For Learning

If you're studying filesystem concepts:
1. **Start with V0.1**: Understand the working implementation in `v0.1/`
2. **Study V2 design**: Read design docs in `knol/` to see modern improvements
3. **Compare**: Understand why V2 makes different design choices
4. **Implement**: Try building V2 yourself following the roadmap
