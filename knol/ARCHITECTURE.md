# PSLFS V2 - System Architecture

**Version:** 2.0
**Status:** Design Phase
**Last Updated:** 2025-10-22

---

## Table of Contents

1. [Overview](#overview)
2. [Layered Architecture](#layered-architecture)
3. [Component Diagram](#component-diagram)
4. [Core Components](#core-components)
5. [Data Flow](#data-flow)
6. [Threading Model](#threading-model)
7. [Error Handling Strategy](#error-handling-strategy)

---

## Overview

PSLFS V2 follows a layered architecture with clear separation between:
- **Storage Layer**: Raw block I/O
- **Filesystem Layer**: Files, directories, metadata
- **Cache Layer**: Performance optimization
- **Interface Layer**: FUSE, CLI tools
- **Security Layer**: Authentication, permissions

```
┌─────────────────────────────────────────────────────┐
│          User Applications / Shell                   │
└─────────────────────────────────────────────────────┘
                         │
          ┌──────────────┴──────────────┐
          ▼                             ▼
┌──────────────────┐          ┌──────────────────┐
│   FUSE Mount     │          │   CLI Tools      │
│   (fuser)        │          │   (clap)         │
└──────────────────┘          └──────────────────┘
          │                             │
          └──────────────┬──────────────┘
                         ▼
          ┌──────────────────────────────┐
          │    Filesystem API Layer      │
          │  (High-level operations)     │
          └──────────────────────────────┘
                         │
          ┌──────────────┴──────────────┐
          ▼                             ▼
┌──────────────────┐          ┌──────────────────┐
│  Metadata Cache  │          │   Permission     │
│  (LRU)           │          │   Manager        │
└──────────────────┘          └──────────────────┘
          │                             │
          └──────────────┬──────────────┘
                         ▼
          ┌──────────────────────────────┐
          │    Core Filesystem Layer     │
          │  (Inodes, directories, etc)  │
          └──────────────────────────────┘
                         │
                         ▼
          ┌──────────────────────────────┐
          │    Block Allocator           │
          │  (Free space management)     │
          └──────────────────────────────┘
                         │
                         ▼
          ┌──────────────────────────────┐
          │    Storage Backend           │
          │  (File, BlockDevice, Memory) │
          └──────────────────────────────┘
                         │
                         ▼
          ┌──────────────────────────────┐
          │    Physical Storage          │
          │  (Disk, Memory, Network)     │
          └──────────────────────────────┘
```

---

## Layered Architecture

### Layer 0: Storage Backend (Abstract)

**Responsibility:** Provide block-level read/write operations

```rust
pub trait StorageBackend: Send + Sync {
    fn read_block(&self, block_id: BlockId) -> Result<Block>;
    fn write_block(&mut self, block_id: BlockId, data: &Block) -> Result<()>;
    fn sync(&mut self) -> Result<()>;
    fn total_blocks(&self) -> u64;
}
```

**Implementations:**
- `FileBackend`: Stores filesystem in regular files (V0.1 compatible)
- `BlockDeviceBackend`: Direct block device access
- `MemoryBackend`: In-memory for testing

**Design Decision:** Abstract backend allows testing and flexibility without changing core logic.

---

### Layer 1: Block Allocator

**Responsibility:** Manage free and allocated blocks

```rust
pub struct BlockAllocator {
    bitmap: RoaringBitmap,  // Efficient sparse bitmap
    total_blocks: u64,
    free_blocks: u64,
}

impl BlockAllocator {
    pub fn allocate(&mut self) -> Result<BlockId>;
    pub fn allocate_contiguous(&mut self, count: usize) -> Result<Vec<BlockId>>;
    pub fn free(&mut self, block_id: BlockId) -> Result<()>;
    pub fn is_free(&self, block_id: BlockId) -> bool;
}
```

**Why Bitmap Instead of Linked List (V0.1)?**
- O(1) allocation via bit scanning
- Compact representation (1 bit per block)
- Easy to persist and recover
- Standard technique in real filesystems

---

### Layer 2: Core Filesystem

**Responsibility:** Implement filesystem abstractions (inodes, directories, files)

#### 2.1 Inode Layer

```rust
pub struct Inode {
    pub id: InodeId,
    pub kind: InodeKind,  // File, Directory, Symlink
    pub size: u64,
    pub blocks: Vec<BlockId>,  // Direct blocks only (keep it simple)
    pub created: SystemTime,
    pub modified: SystemTime,
    pub permissions: Permissions,
    pub owner: UserId,
}

pub enum InodeKind {
    File,
    Directory,
    // Symlink(PathBuf),  // Future
}
```

**Design Decision:**
- Direct block pointers only (no indirect blocks initially)
- Maximum file size: ~16MB (4096 blocks × 4KB)
- Simple but sufficient for educational use
- Can extend later with indirect blocks as "advanced feature"

#### 2.2 Directory Layer

```rust
pub struct Directory {
    pub inode: InodeId,
    pub entries: BTreeMap<String, InodeId>,  // Sorted for iteration
    pub parent: Option<InodeId>,
}

impl Directory {
    pub fn lookup(&self, name: &str) -> Option<InodeId>;
    pub fn insert(&mut self, name: String, inode: InodeId) -> Result<()>;
    pub fn remove(&mut self, name: &str) -> Result<InodeId>;
    pub fn list(&self) -> Vec<(String, InodeId)>;
}
```

**Why BTreeMap?**
- Ordered iteration (important for `ls`)
- O(log n) lookup (better than V0.1's O(n) linked list)
- Simple to understand and use
- Good cache locality

---

### Layer 3: Metadata Cache

**Responsibility:** Cache hot inodes and directories

```rust
pub struct MetadataCache {
    inodes: LruCache<InodeId, Inode>,
    directories: LruCache<InodeId, Directory>,
    max_size: usize,
}

impl MetadataCache {
    pub fn get_inode(&mut self, id: InodeId) -> Option<&Inode>;
    pub fn put_inode(&mut self, inode: Inode);
    pub fn invalidate(&mut self, id: InodeId);
    pub fn flush(&mut self) -> Result<()>;  // Write dirty entries
}
```

**Cache Strategy:**
- LRU eviction (simple and effective)
- Write-through (simpler than write-back)
- Configurable size (default: 1000 entries)

---

### Layer 4: Filesystem API

**Responsibility:** High-level operations

```rust
pub struct Filesystem {
    backend: Box<dyn StorageBackend>,
    allocator: BlockAllocator,
    cache: MetadataCache,
    auth: AuthManager,
}

impl Filesystem {
    // File operations
    pub fn create_file(&mut self, parent: InodeId, name: &str) -> Result<InodeId>;
    pub fn read_file(&self, inode: InodeId, offset: u64, buf: &mut [u8]) -> Result<usize>;
    pub fn write_file(&mut self, inode: InodeId, offset: u64, data: &[u8]) -> Result<usize>;
    pub fn delete_file(&mut self, parent: InodeId, name: &str) -> Result<()>;

    // Directory operations
    pub fn create_dir(&mut self, parent: InodeId, name: &str) -> Result<InodeId>;
    pub fn read_dir(&self, inode: InodeId) -> Result<Vec<DirEntry>>;
    pub fn delete_dir(&mut self, parent: InodeId, name: &str) -> Result<()>;

    // Metadata operations
    pub fn stat(&self, inode: InodeId) -> Result<Metadata>;
    pub fn chmod(&mut self, inode: InodeId, perms: Permissions) -> Result<()>;
    pub fn chown(&mut self, inode: InodeId, owner: UserId) -> Result<()>;
}
```

---

### Layer 5: Interface Layers

#### 5.1 FUSE Layer

```rust
pub struct PslfsFuse {
    fs: Arc<Mutex<Filesystem>>,
}

impl Filesystem for PslfsFuse {
    // Implement fuser::Filesystem trait
    fn lookup(&mut self, req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry);
    fn getattr(&mut self, req: &Request, ino: u64, reply: ReplyAttr);
    fn read(&mut self, req: &Request, ino: u64, fh: u64, offset: i64, size: u32, reply: ReplyData);
    fn write(&mut self, req: &Request, ino: u64, fh: u64, offset: i64, data: &[u8], reply: ReplyWrite);
    // ... other FUSE operations
}
```

#### 5.2 CLI Layer

```bash
# pslfs command structure
pslfs init <path>           # Create new filesystem
pslfs mount <path> <mnt>    # Mount via FUSE
pslfs ls <path>             # List directory (direct access)
pslfs cat <path>:<file>     # Read file (direct access)
pslfs check <path>          # Fsck - check consistency
pslfs stats <path>          # Show statistics
```

---

## Core Components

### Component 1: Superblock

**Purpose:** Store filesystem metadata

```rust
#[derive(Serialize, Deserialize)]
pub struct Superblock {
    pub magic: u32,              // 0x50534C46 ("PSLF")
    pub version: u32,            // Format version
    pub block_size: u32,         // Usually 4096
    pub total_blocks: u64,
    pub free_blocks: u64,
    pub inode_count: u64,
    pub root_inode: InodeId,
    pub created: SystemTime,
    pub last_mounted: SystemTime,
    pub mount_count: u32,
    pub state: FsState,          // Clean, Dirty, Error
}
```

**Location:** Always at block 0

---

### Component 2: Inode Table

**Purpose:** Store all inodes

**Layout:**
```
Block 1-N: Inode bitmap (which inodes are allocated)
Block N+1 onwards: Actual inodes (packed)
```

**Calculation:**
```rust
const INODES_PER_BLOCK: usize = BLOCK_SIZE / size_of::<Inode>();
fn inode_block_id(inode_id: InodeId) -> BlockId {
    INODE_TABLE_START + (inode_id / INODES_PER_BLOCK)
}
```

---

### Component 3: Data Blocks

**Purpose:** Store file contents and directory data

- **File data blocks:** Raw file content
- **Directory blocks:** Serialized `BTreeMap<String, InodeId>`

---

### Component 4: Authentication Manager

```rust
pub struct AuthManager {
    users: HashMap<UserId, User>,
    sessions: HashMap<SessionId, UserId>,
}

pub struct User {
    pub id: UserId,
    pub username: String,
    pub password_hash: [u8; 32],  // bcrypt or argon2
    pub created: SystemTime,
}

impl AuthManager {
    pub fn create_user(&mut self, username: &str, password: &str) -> Result<UserId>;
    pub fn authenticate(&mut self, username: &str, password: &str) -> Result<SessionId>;
    pub fn check_permission(&self, session: SessionId, inode: &Inode, perm: Permission) -> bool;
}
```

**Improvement over V0.1:** Use proper password hashing (bcrypt/argon2), not simple character shift

---

## Data Flow

### Example: Creating a File

```
1. User: `touch /mnt/pslfs/hello.txt`
   ↓
2. FUSE: create("/hello.txt", mode=0644)
   ↓
3. Filesystem::create_file(root_inode, "hello.txt")
   ↓
4. Steps:
   a) Check permission on parent directory
   b) Allocate new inode ID
   c) Allocate inode block
   d) Create Inode struct (size=0, blocks=[], ...)
   e) Write inode to block
   f) Read parent directory from cache/disk
   g) Insert "hello.txt" -> inode_id into directory BTreeMap
   h) Serialize and write directory back
   i) Update bitmap: mark inode block as allocated
   j) Update superblock: increment inode_count
   k) Sync all writes to disk
   ↓
5. Return success to FUSE
   ↓
6. FUSE returns to kernel
   ↓
7. User sees file created
```

### Example: Reading a File

```
1. User: `cat /mnt/pslfs/hello.txt`
   ↓
2. FUSE:
   - lookup("hello.txt") -> get inode ID
   - getattr(inode) -> get file size
   - read(inode, offset=0, size=4096)
   ↓
3. Filesystem::read_file(inode_id, 0, buf)
   ↓
4. Steps:
   a) Get inode from cache or disk
   b) Check read permission
   c) Calculate which blocks to read
   d) For each block:
      - Read block from storage
      - Copy relevant bytes to buffer
   e) Return bytes read
   ↓
5. FUSE returns data to kernel
   ↓
6. User sees file content
```

---

## Threading Model

### Single-Threaded Core (Initially)

```rust
pub struct Filesystem {
    // No Arc<Mutex<...>> internally
    // Caller manages concurrency
}
```

**Rationale:**
- Simpler to understand and debug
- FUSE can handle this with external locking
- Can add internal parallelism later

### FUSE Threading

```rust
pub struct PslfsFuse {
    fs: Arc<Mutex<Filesystem>>,  // One lock for entire FS
}
```

**Improvement Path:**
1. Start: Single global lock
2. Later: Reader-writer lock
3. Advanced: Fine-grained locking per inode

---

## Error Handling Strategy

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Not a directory: {0}")]
    NotADirectory(InodeId),

    #[error("Not a file: {0}")]
    NotAFile(InodeId),

    #[error("Directory not empty: {0}")]
    DirectoryNotEmpty(InodeId),

    #[error("No space left on device")]
    NoSpace,

    #[error("Filesystem corrupted: {0}")]
    Corrupted(String),
}

pub type Result<T> = std::result::Result<T, FsError>;
```

### Error Handling Rules

1. **Use `Result` everywhere** - No panics in normal operation
2. **Context-rich errors** - Include inode IDs, paths, etc.
3. **Fail fast on corruption** - Don't try to recover from invalid data
4. **Log all errors** - Use `log` crate
5. **Convert at boundaries** - `FsError -> errno` for FUSE

---

## Module Structure

```
pslfs/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   │
│   ├── storage/
│   │   ├── mod.rs           # StorageBackend trait
│   │   ├── file.rs          # FileBackend impl
│   │   ├── memory.rs        # MemoryBackend impl
│   │   └── block.rs         # Block type
│   │
│   ├── allocator/
│   │   ├── mod.rs           # BlockAllocator
│   │   └── bitmap.rs        # Bitmap implementation
│   │
│   ├── core/
│   │   ├── mod.rs           # Core exports
│   │   ├── inode.rs         # Inode structure
│   │   ├── directory.rs     # Directory structure
│   │   ├── superblock.rs    # Superblock
│   │   └── permissions.rs   # Permission types
│   │
│   ├── cache/
│   │   └── mod.rs           # MetadataCache
│   │
│   ├── fs/
│   │   ├── mod.rs           # Filesystem struct
│   │   ├── file_ops.rs      # File operations
│   │   └── dir_ops.rs       # Directory operations
│   │
│   ├── auth/
│   │   ├── mod.rs           # AuthManager
│   │   └── user.rs          # User types
│   │
│   ├── fuse/
│   │   ├── mod.rs           # FUSE implementation
│   │   └── conversion.rs    # Error/type conversions
│   │
│   ├── cli/
│   │   ├── mod.rs           # CLI command parsing
│   │   ├── init.rs          # pslfs init
│   │   ├── mount.rs         # pslfs mount
│   │   └── check.rs         # pslfs check
│   │
│   └── error.rs             # FsError type
│
└── tests/
    ├── integration/         # Integration tests
    ├── fuzz/                # Fuzz tests
    └── property/            # Property-based tests
```

---

## Design Decisions Summary

| Decision | Rationale |
|----------|-----------|
| Rust over C | Memory safety, modern tooling, better error handling |
| Bitmap vs linked list | O(1) allocation, compact, standard approach |
| BTreeMap for directories | Ordered, O(log n), simple |
| Direct blocks only | Keep it simple, sufficient for learning |
| LRU cache | Simple, effective, well-understood |
| Write-through cache | Simpler than write-back, less chance of corruption |
| Single global lock | Start simple, optimize later |
| Abstract storage backend | Testability, flexibility |
| No journaling | Keep it simple, crash recovery is advanced topic |

---

## Performance Targets

| Operation | V0.1 | V2 Target | How |
|-----------|------|-----------|-----|
| Read 1MB file | 25s | 0.5s | Larger blocks, caching |
| Create file | 2.2ms | 0.5ms | Bitmap allocation, write-through cache |
| List 100 files | 30ms | 3ms | BTreeMap, cached directories |
| Random read | O(n) | O(log n) | Indexed structures |

**Still 10-50x slower than ext4, but 10-50x faster than V0.1!**

---

## Future Extensions (Post-MVP)

1. **Indirect blocks** - Support larger files
2. **Snapshots** - Copy-on-write at block level
3. **Compression** - Per-file compression flag
4. **Extended attributes** - Metadata key-value store
5. **Symbolic links** - Path redirection
6. **Hard links** - Multiple directory entries for one inode
7. **Write-back caching** - Better write performance
8. **Fine-grained locking** - Better concurrency

Each extension is a self-contained learning module!
