# PSLFS V2 - Data Structures Specification

**Version:** 2.0
**Status:** Design Phase
**Last Updated:** 2025-10-22

---

## Table of Contents

1. [Overview](#overview)
2. [Constants and Configuration](#constants-and-configuration)
3. [On-Disk Structures](#on-disk-structures)
4. [In-Memory Structures](#in-memory-structures)
5. [Serialization Format](#serialization-format)
6. [Disk Layout](#disk-layout)
7. [Example Walkthrough](#example-walkthrough)

---

## Overview

PSLFS V2 uses a simple, predictable on-disk format that can be inspected with standard tools. All structures are serialized using `bincode` (binary format) or `serde_json` (for debugging).

**Design Principles:**
- Fixed-size structures where possible
- Explicit padding for alignment
- Version fields for future compatibility
- Checksums for corruption detection

---

## Constants and Configuration

```rust
/// Core filesystem constants
pub mod constants {
    use std::num::NonZeroUsize;

    /// Magic number: "PSLF" in ASCII
    pub const MAGIC: u32 = 0x50534C46;

    /// Current format version
    pub const VERSION: u32 = 2;

    /// Default block size (4KB)
    pub const BLOCK_SIZE: usize = 4096;

    /// Maximum filename length (bytes)
    pub const MAX_FILENAME_LEN: usize = 255;

    /// Maximum path length (bytes)
    pub const MAX_PATH_LEN: usize = 4096;

    /// Number of direct blocks per inode
    pub const DIRECT_BLOCKS: usize = 12;

    /// Root inode ID (always 1)
    pub const ROOT_INODE: InodeId = InodeId(1);

    /// Reserved inodes
    pub const RESERVED_INODES: u64 = 10;

    /// Superblock location (always block 0)
    pub const SUPERBLOCK_BLOCK: BlockId = BlockId(0);

    /// Default cache size (number of entries)
    pub const DEFAULT_CACHE_SIZE: NonZeroUsize =
        unsafe { NonZeroUsize::new_unchecked(1000) };
}
```

---

## On-Disk Structures

### 1. Superblock (Block 0)

```rust
use serde::{Serialize, Deserialize};

/// Superblock stores global filesystem metadata
/// Size: ~256 bytes (rest of block is reserved)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct Superblock {
    /// Magic number for format detection
    pub magic: u32,

    /// Format version
    pub version: u32,

    /// Block size in bytes
    pub block_size: u32,

    /// Total number of blocks
    pub total_blocks: u64,

    /// Number of free blocks
    pub free_blocks: u64,

    /// Total number of inodes
    pub total_inodes: u64,

    /// Number of allocated inodes
    pub allocated_inodes: u64,

    /// Root directory inode
    pub root_inode: InodeId,

    /// Filesystem UUID
    pub uuid: [u8; 16],

    /// Volume label (null-terminated)
    pub label: [u8; 64],

    /// Creation timestamp (seconds since epoch)
    pub created: u64,

    /// Last mount timestamp
    pub last_mounted: u64,

    /// Last modified timestamp
    pub last_modified: u64,

    /// Mount count
    pub mount_count: u32,

    /// Maximum mount count before check
    pub max_mount_count: u32,

    /// Filesystem state
    pub state: FsState,

    /// Block where inode bitmap starts
    pub inode_bitmap_block: BlockId,

    /// Number of blocks for inode bitmap
    pub inode_bitmap_blocks: u32,

    /// Block where data bitmap starts
    pub data_bitmap_block: BlockId,

    /// Number of blocks for data bitmap
    pub data_bitmap_blocks: u32,

    /// Block where inode table starts
    pub inode_table_block: BlockId,

    /// Number of blocks for inode table
    pub inode_table_blocks: u32,

    /// First data block
    pub first_data_block: BlockId,

    /// Checksum of this superblock (CRC32)
    pub checksum: u32,

    /// Reserved for future use
    pub reserved: [u8; 100],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum FsState {
    /// Filesystem is clean
    Clean = 0,

    /// Filesystem is mounted
    Mounted = 1,

    /// Filesystem has errors
    Error = 2,

    /// Filesystem needs check
    NeedsCheck = 3,
}

impl Superblock {
    /// Calculate and update checksum
    pub fn update_checksum(&mut self) {
        self.checksum = 0; // Zero out before calculating
        let bytes = bincode::serialize(self).unwrap();
        self.checksum = crc32fast::hash(&bytes);
    }

    /// Verify checksum
    pub fn verify_checksum(&self) -> bool {
        let stored = self.checksum;
        let mut temp = self.clone();
        temp.update_checksum();
        temp.checksum == stored
    }
}
```

---

### 2. Inode

```rust
/// Unique inode identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct InodeId(pub u64);

/// On-disk inode structure
/// Size: 256 bytes (fixed)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct Inode {
    /// Inode number
    pub id: InodeId,

    /// Type of inode
    pub kind: InodeKind,

    /// File mode (permissions)
    pub mode: u16,

    /// Owner user ID
    pub uid: u32,

    /// Owner group ID
    pub gid: u32,

    /// File size in bytes
    pub size: u64,

    /// Number of blocks allocated
    pub blocks: u32,

    /// Number of hard links
    pub nlink: u32,

    /// Creation time (seconds since epoch)
    pub created: u64,

    /// Last modification time
    pub modified: u64,

    /// Last access time
    pub accessed: u64,

    /// Direct block pointers (12 blocks = 48KB max)
    pub direct: [BlockId; DIRECT_BLOCKS],

    /// Indirect block pointer (for future extension)
    pub indirect: BlockId,

    /// Double indirect block pointer (for future extension)
    pub double_indirect: BlockId,

    /// Triple indirect block pointer (for future extension)
    pub triple_indirect: BlockId,

    /// Inode flags
    pub flags: InodeFlags,

    /// Checksum (CRC32)
    pub checksum: u32,

    /// Reserved for future use
    pub reserved: [u8; 40],
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum InodeKind {
    File = 1,
    Directory = 2,
    Symlink = 3,
}

bitflags::bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct InodeFlags: u32 {
        /// Inode is immutable
        const IMMUTABLE = 0b00000001;

        /// Inode is append-only
        const APPEND_ONLY = 0b00000010;

        /// Inode data is compressed
        const COMPRESSED = 0b00000100;

        /// Inode is encrypted
        const ENCRYPTED = 0b00001000;
    }
}

impl Inode {
    /// Create a new inode
    pub fn new(id: InodeId, kind: InodeKind, mode: u16, uid: u32, gid: u32) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            kind,
            mode,
            uid,
            gid,
            size: 0,
            blocks: 0,
            nlink: match kind {
                InodeKind::Directory => 2, // . and ..
                _ => 1,
            },
            created: now,
            modified: now,
            accessed: now,
            direct: [BlockId(0); DIRECT_BLOCKS],
            indirect: BlockId(0),
            double_indirect: BlockId(0),
            triple_indirect: BlockId(0),
            flags: InodeFlags::empty(),
            checksum: 0,
            reserved: [0; 40],
        }
    }

    /// Get block at given index
    pub fn get_block(&self, index: usize) -> Option<BlockId> {
        if index < DIRECT_BLOCKS {
            let block = self.direct[index];
            if block.0 != 0 {
                Some(block)
            } else {
                None
            }
        } else {
            // TODO: Implement indirect blocks
            None
        }
    }

    /// Set block at given index
    pub fn set_block(&mut self, index: usize, block: BlockId) -> Result<()> {
        if index < DIRECT_BLOCKS {
            self.direct[index] = block;
            Ok(())
        } else {
            // TODO: Implement indirect blocks
            Err(FsError::NoSpace)
        }
    }
}
```

---

### 3. Directory Entry

```rust
/// On-disk directory entry
/// Size: variable (serialized with bincode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    /// Inode number
    pub inode: InodeId,

    /// Entry type
    pub kind: InodeKind,

    /// Filename (UTF-8)
    pub name: String,
}

/// Directory data (stored in inode data blocks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryData {
    /// Parent inode (0 for root)
    pub parent: InodeId,

    /// Map of filename -> inode
    /// Using BTreeMap for ordered iteration
    pub entries: BTreeMap<String, DirEntry>,
}

impl DirectoryData {
    /// Create new empty directory
    pub fn new(parent: InodeId, self_inode: InodeId) -> Self {
        let mut entries = BTreeMap::new();

        // Add . and .. entries
        entries.insert(".".to_string(), DirEntry {
            inode: self_inode,
            kind: InodeKind::Directory,
            name: ".".to_string(),
        });

        if parent.0 != 0 {
            entries.insert("..".to_string(), DirEntry {
                inode: parent,
                kind: InodeKind::Directory,
                name: "..".to_string(),
            });
        }

        Self { parent, entries }
    }

    /// Add entry
    pub fn add(&mut self, name: String, inode: InodeId, kind: InodeKind) -> Result<()> {
        if self.entries.contains_key(&name) {
            return Err(FsError::AlreadyExists(name));
        }
        self.entries.insert(name.clone(), DirEntry { inode, kind, name });
        Ok(())
    }

    /// Remove entry
    pub fn remove(&mut self, name: &str) -> Result<InodeId> {
        self.entries
            .remove(name)
            .map(|entry| entry.inode)
            .ok_or_else(|| FsError::NotFound(name.to_string()))
    }

    /// Lookup entry
    pub fn lookup(&self, name: &str) -> Option<&DirEntry> {
        self.entries.get(name)
    }
}
```

---

### 4. Block Types

```rust
/// Block identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct BlockId(pub u64);

impl BlockId {
    /// Null block (invalid)
    pub const NULL: BlockId = BlockId(0);

    pub fn is_null(self) -> bool {
        self.0 == 0
    }
}

/// Raw block data
pub struct Block {
    pub data: [u8; BLOCK_SIZE],
}

impl Block {
    pub fn new() -> Self {
        Self {
            data: [0; BLOCK_SIZE],
        }
    }

    pub fn from_slice(data: &[u8]) -> Self {
        let mut block = Self::new();
        let len = data.len().min(BLOCK_SIZE);
        block.data[..len].copy_from_slice(&data[..len]);
        block
    }
}
```

---

### 5. Bitmap

```rust
use bitvec::prelude::*;

/// Bitmap for tracking allocated blocks/inodes
pub struct Bitmap {
    /// Bit vector (1 = allocated, 0 = free)
    bits: BitVec<u8, Lsb0>,

    /// Total number of items
    total: usize,

    /// Number of free items
    free: usize,
}

impl Bitmap {
    pub fn new(total: usize) -> Self {
        Self {
            bits: bitvec![0; total],
            total,
            free: total,
        }
    }

    /// Allocate a free item
    pub fn allocate(&mut self) -> Option<usize> {
        for (i, bit) in self.bits.iter_mut().enumerate() {
            if !*bit {
                bit.set(true);
                self.free -= 1;
                return Some(i);
            }
        }
        None
    }

    /// Free an item
    pub fn free(&mut self, index: usize) -> Result<()> {
        if index >= self.total {
            return Err(FsError::InvalidBlock(index));
        }
        if !self.bits[index] {
            return Err(FsError::DoubleFree(index));
        }
        self.bits.set(index, false);
        self.free += 1;
        Ok(())
    }

    /// Check if item is allocated
    pub fn is_allocated(&self, index: usize) -> bool {
        index < self.total && self.bits[index]
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.bits.as_raw_slice().to_vec()
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8], total: usize) -> Self {
        let bits = BitVec::from_slice(bytes);
        let free = bits.iter().filter(|b| !**b).count();
        Self { bits, total, free }
    }
}
```

---

### 6. User and Authentication

```rust
/// User ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct UserId(pub u32);

/// User information (stored in separate authentication file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub password_hash: [u8; 32],  // bcrypt hash
    pub created: u64,
    pub last_login: u64,
}

/// Permissions (Unix-style)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Permissions {
    pub mode: u16,
}

impl Permissions {
    pub fn new(mode: u16) -> Self {
        Self { mode }
    }

    pub fn can_read(&self, uid: UserId, inode_uid: u32) -> bool {
        if uid.0 == inode_uid {
            self.mode & 0o400 != 0  // Owner read
        } else {
            self.mode & 0o004 != 0  // Other read
        }
    }

    pub fn can_write(&self, uid: UserId, inode_uid: u32) -> bool {
        if uid.0 == inode_uid {
            self.mode & 0o200 != 0  // Owner write
        } else {
            self.mode & 0o002 != 0  // Other write
        }
    }

    pub fn can_execute(&self, uid: UserId, inode_uid: u32) -> bool {
        if uid.0 == inode_uid {
            self.mode & 0o100 != 0  // Owner execute
        } else {
            self.mode & 0o001 != 0  // Other execute
        }
    }
}
```

---

## In-Memory Structures

### 1. Metadata Cache

```rust
use lru::LruCache;

pub struct MetadataCache {
    /// Cache of inodes
    inodes: LruCache<InodeId, Inode>,

    /// Cache of directory data
    directories: LruCache<InodeId, DirectoryData>,

    /// Dirty flags
    dirty_inodes: HashSet<InodeId>,
    dirty_directories: HashSet<InodeId>,
}

impl MetadataCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            inodes: LruCache::new(capacity.try_into().unwrap()),
            directories: LruCache::new(capacity.try_into().unwrap()),
            dirty_inodes: HashSet::new(),
            dirty_directories: HashSet::new(),
        }
    }

    pub fn get_inode(&mut self, id: InodeId) -> Option<&Inode> {
        self.inodes.get(&id)
    }

    pub fn put_inode(&mut self, inode: Inode, dirty: bool) {
        let id = inode.id;
        self.inodes.put(id, inode);
        if dirty {
            self.dirty_inodes.insert(id);
        }
    }

    pub fn flush_dirty(&mut self) -> Vec<(InodeId, Inode)> {
        let mut to_flush = Vec::new();
        for id in self.dirty_inodes.drain() {
            if let Some(inode) = self.inodes.get(&id) {
                to_flush.push((id, inode.clone()));
            }
        }
        to_flush
    }
}
```

---

## Serialization Format

### Using Bincode

```rust
use bincode::{serialize, deserialize};

// Serialize inode to bytes
let inode = Inode::new(...);
let bytes: Vec<u8> = serialize(&inode)?;

// Deserialize inode from bytes
let inode: Inode = deserialize(&bytes)?;
```

### Why Bincode?

- **Fast**: Zero-copy where possible
- **Compact**: More efficient than JSON
- **Versioned**: Can detect format changes
- **Rust-native**: Works with `serde` traits

### Debug Format (JSON)

```rust
// For debugging: pretty-print as JSON
let json = serde_json::to_string_pretty(&inode)?;
println!("{}", json);
```

---

## Disk Layout

```
╔══════════════════════════════════════════════════════╗
║  Block 0: Superblock                                  ║
╠══════════════════════════════════════════════════════╣
║  Block 1-N: Inode Bitmap                              ║
║  (1 bit per inode, tracks allocation)                 ║
╠══════════════════════════════════════════════════════╣
║  Block N+1 to M: Data Block Bitmap                    ║
║  (1 bit per data block, tracks allocation)            ║
╠══════════════════════════════════════════════════════╣
║  Block M+1 to P: Inode Table                          ║
║  (Fixed-size array of inodes)                         ║
║  - Each block contains 16 inodes (256 bytes each)     ║
║  - Inode 1 = root directory                           ║
╠══════════════════════════════════════════════════════╣
║  Block P+1 onwards: Data Blocks                       ║
║  - File contents                                      ║
║  - Directory data (serialized BTreeMap)               ║
║  - Future: indirect block pointers                    ║
╚══════════════════════════════════════════════════════╝
```

### Size Calculations (Example: 1GB filesystem)

```rust
const BLOCK_SIZE: u64 = 4096;
const TOTAL_SIZE: u64 = 1024 * 1024 * 1024;  // 1GB
const TOTAL_BLOCKS: u64 = TOTAL_SIZE / BLOCK_SIZE;  // 262,144 blocks

// Superblock: 1 block
const SUPERBLOCK_BLOCKS: u64 = 1;

// Inode bitmap: 1 bit per inode, let's say 64K inodes
const MAX_INODES: u64 = 65536;
const INODE_BITMAP_BLOCKS: u64 =
    (MAX_INODES / 8 + BLOCK_SIZE - 1) / BLOCK_SIZE;  // 2 blocks

// Data bitmap: 1 bit per data block
const DATA_BITMAP_BLOCKS: u64 =
    (TOTAL_BLOCKS / 8 + BLOCK_SIZE - 1) / BLOCK_SIZE;  // 8 blocks

// Inode table: 256 bytes per inode, 16 inodes per block
const INODES_PER_BLOCK: u64 = BLOCK_SIZE / 256;  // 16
const INODE_TABLE_BLOCKS: u64 =
    (MAX_INODES + INODES_PER_BLOCK - 1) / INODES_PER_BLOCK;  // 4096 blocks

// Data blocks: everything else
const DATA_BLOCKS: u64 = TOTAL_BLOCKS
    - SUPERBLOCK_BLOCKS
    - INODE_BITMAP_BLOCKS
    - DATA_BITMAP_BLOCKS
    - INODE_TABLE_BLOCKS;  // ~258,000 blocks = ~1GB
```

---

## Example Walkthrough

### Creating a 10MB Filesystem

```rust
// 1. Create filesystem
let total_blocks = (10 * 1024 * 1024) / 4096;  // 2560 blocks
let max_inodes = 1024;

// 2. Initialize superblock
let mut sb = Superblock {
    magic: MAGIC,
    version: VERSION,
    block_size: 4096,
    total_blocks,
    free_blocks: total_blocks - overhead,
    total_inodes: max_inodes,
    allocated_inodes: 1,  // Root
    root_inode: ROOT_INODE,
    uuid: Uuid::new_v4().as_bytes(),
    // ... rest initialized
};

// 3. Layout calculation
let inode_bitmap_blocks = (max_inodes / 8 + 4095) / 4096;  // 1 block
let data_bitmap_blocks = (total_blocks / 8 + 4095) / 4096;  // 1 block
let inode_table_blocks = (max_inodes * 256 + 4095) / 4096;  // 64 blocks

sb.inode_bitmap_block = BlockId(1);
sb.inode_bitmap_blocks = inode_bitmap_blocks;
sb.data_bitmap_block = BlockId(1 + inode_bitmap_blocks);
sb.data_bitmap_blocks = data_bitmap_blocks;
sb.inode_table_block = BlockId(1 + inode_bitmap_blocks + data_bitmap_blocks);
sb.inode_table_blocks = inode_table_blocks;
sb.first_data_block = BlockId(1 + inode_bitmap_blocks + data_bitmap_blocks + inode_table_blocks);

// 4. Create root inode
let root = Inode::new(ROOT_INODE, InodeKind::Directory, 0o755, 0, 0);

// 5. Create root directory data
let root_dir = DirectoryData::new(InodeId(0), ROOT_INODE);

// 6. Write to disk
write_block(0, &serialize(&sb)?)?;
write_inode(ROOT_INODE, &root)?;
write_directory_data(ROOT_INODE, &root_dir)?;
```

### Reading a File: `/home/user/document.txt`

```
1. Start at root (inode 1)
2. Read root directory data
3. Lookup "home" -> inode 42
4. Read inode 42 (verify it's a directory)
5. Read directory data for inode 42
6. Lookup "user" -> inode 156
7. Read inode 156 (verify it's a directory)
8. Read directory data for inode 156
9. Lookup "document.txt" -> inode 789
10. Read inode 789 (verify it's a file)
11. Read data blocks from inode 789's block list
12. Return file contents
```

---

## Memory Footprint Estimates

```
Superblock: 256 bytes (in memory once)
Inode: 256 bytes each
DirectoryData: variable (100-10000 bytes typical)

Cache with 1000 entries:
- 1000 inodes = 256KB
- 1000 directories = ~1-10MB (depends on size)
- Total: ~1-10MB RAM

Minimal operation (no cache):
- ~1MB RAM for core structures
```

---

## Compatibility Notes

### V0.1 Migration

To read V0.1 filesystems:

```rust
// Detect format
let magic = read_u32(0)?;
if magic == OLD_MAGIC {
    // Read V0.1 structures (linked lists)
    // Convert to V2 format
    migrate_v01_to_v2()?;
}
```

### Future Versions

All structures have:
- Version field
- Reserved space for extensions
- Checksums for validation

Adding new fields:
```rust
// V2
pub struct Inode {
    // ... existing fields
    pub reserved: [u8; 40],  // Can use this later
}

// V3 (future)
pub struct Inode {
    // ... existing fields
    pub new_feature: u64,      // Use 8 bytes of reserved
    pub reserved: [u8; 32],    // Still have space
}
```

---

This completes the data structures specification. All structures are designed to be:
- **Simple**: Easy to understand and implement
- **Efficient**: Reasonable space and time complexity
- **Safe**: Checksums and validation
- **Extensible**: Room for future features
