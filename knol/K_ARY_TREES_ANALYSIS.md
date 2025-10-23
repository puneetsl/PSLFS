# K-ary Trees in Filesystems - Analysis for PSLFS V2

**Should PSLFS use K-ary trees for files and folders?**

**Short Answer:** Yes! And we already do (for directories).

---

## Table of Contents

1. [What are K-ary Trees?](#what-are-k-ary-trees)
2. [Current V2 Design](#current-v2-design)
3. [K-ary Trees for Directories](#k-ary-trees-for-directories)
4. [K-ary Trees for Files](#k-ary-trees-for-files)
5. [Real-World Examples](#real-world-examples)
6. [Recommendations](#recommendations)

---

## What are K-ary Trees?

### Definition

A **K-ary tree** is a tree where each node can have up to **K children**.

```
Binary Tree (K=2):        K-ary Tree (K=4):
       A                         A
      / \                    / | | \
     B   C                  B  C D  E
    / \                    /|  |\
   D   E                  F G  H I
```

### Why K-ary for Filesystems?

**Disk I/O is expensive!** Reading one block costs about the same as reading 4KB of data. So:

- **Binary tree (K=2)**: Many small nodes → Many disk reads
- **K-ary tree (K=128)**: Fewer large nodes → Fewer disk reads

**Example: Finding 1 entry among 1,000,000:**

| Tree Type | K | Height | Disk Reads |
|-----------|---|--------|------------|
| Binary | 2 | ~20 | 20 reads |
| 16-ary | 16 | ~5 | 5 reads |
| B-tree | 128 | ~3 | 3 reads |

**6x fewer disk reads with B-tree!**

---

## Current V2 Design

### For Directories: BTreeMap (Already K-ary!)

```rust
use std::collections::BTreeMap;

pub struct Directory {
    pub inode: InodeId,
    pub entries: BTreeMap<String, InodeId>,  // ← B-tree (K-ary)
    pub parent: Option<InodeId>,
}
```

**What is BTreeMap?**
- B-tree implementation from Rust standard library
- Self-balancing K-ary tree
- K typically 32-128 (implementation detail)
- O(log n) insert, delete, lookup
- **Ordered iteration** (important for `ls` command!)

**Why this is good:**
- ✅ Efficient for large directories (1000s of files)
- ✅ Ordered iteration (sorted directory listings)
- ✅ O(log n) operations
- ✅ Simple to use (standard library)
- ✅ Well-tested

**Alternative was HashMap:**
```rust
pub entries: HashMap<String, InodeId>,  // O(1) but unordered
```
- Faster lookup (O(1) vs O(log n))
- But no ordering (ugly `ls` output)
- More complex to serialize

**Decision: BTreeMap is better for educational filesystem**

---

### For Files: Direct + Indirect Blocks

```rust
pub struct Inode {
    // Direct blocks (simple array)
    pub direct: [BlockId; 12],           // 12 × 4KB = 48KB

    // Indirect blocks (K-ary trees)
    pub indirect: BlockId,               // 1024 blocks = 4MB
    pub double_indirect: BlockId,        // 1M blocks = 4GB
    pub triple_indirect: BlockId,        // 1T blocks = 4TB
}
```

**Structure of indirect blocks:**

```
Direct blocks (K=12, simple array):
  Inode → [Block0, Block1, ..., Block11]

Indirect block (K=1024, 1-level tree):
  Inode → Indirect → [Block0, Block1, ..., Block1023]

Double indirect (K=1024, 2-level tree):
  Inode → DoubleIndirect → [Indirect0, Indirect1, ..., Indirect1023]
                               ↓
                            [Block0, Block1, ..., Block1023]

Triple indirect (K=1024, 3-level tree):
  Inode → TripleIndirect → [DoubleIndirect0, DoubleIndirect1, ...]
                               ↓
                            [Indirect0, Indirect1, ...]
                               ↓
                            [Block0, Block1, ...]
```

**This is a K-ary tree where K = BLOCK_SIZE / sizeof(BlockId) = 4096 / 8 = 512!**

---

## K-ary Trees for Directories

### Option 1: BTreeMap (Current Design) ✅

**Structure:**
```rust
pub struct Directory {
    pub entries: BTreeMap<String, InodeId>,
}
```

**Pros:**
- ✅ Simple to implement (standard library)
- ✅ Self-balancing
- ✅ Ordered iteration
- ✅ O(log n) operations
- ✅ Good for educational purposes

**Cons:**
- ❌ In-memory only (need to serialize entire directory)
- ❌ Large directories consume more memory
- ❌ Not optimized for disk I/O

**When to use:**
- Directories with < 10,000 entries
- Educational filesystems
- Simplicity matters

**Serialization:**
```rust
// Serialize entire BTreeMap to blocks
let dir_data = bincode::serialize(&directory)?;
let blocks_needed = (dir_data.len() + BLOCK_SIZE - 1) / BLOCK_SIZE;

// Write to disk
for (i, chunk) in dir_data.chunks(BLOCK_SIZE).enumerate() {
    write_block(inode.direct[i], chunk)?;
}
```

---

### Option 2: On-Disk B-tree (Advanced)

**Structure:**
```rust
pub struct BTreeNode {
    pub is_leaf: bool,
    pub num_keys: usize,
    pub keys: Vec<String>,           // Filenames
    pub values: Vec<InodeId>,        // Inode numbers (leaf only)
    pub children: Vec<BlockId>,      // Child nodes (internal only)
}

pub struct Directory {
    pub root_block: BlockId,  // Root of B-tree
}
```

**Example 4-ary B-tree:**
```
Block 100 (Internal Node):
  keys: ["d", "m", "s"]
  children: [Block101, Block102, Block103, Block104]

Block 101 (Leaf):
  keys: ["a.txt", "b.txt", "c.txt"]
  values: [Inode5, Inode7, Inode9]

Block 102 (Leaf):
  keys: ["e.txt", "h.txt", "l.txt"]
  values: [Inode12, Inode15, Inode18]
```

**Pros:**
- ✅ Handles very large directories (millions of files)
- ✅ Only loads needed nodes
- ✅ Good cache locality
- ✅ Used by production filesystems

**Cons:**
- ❌ Complex to implement
- ❌ Need to handle node splitting/merging
- ❌ More disk I/O for small directories
- ❌ Harder to debug

**When to use:**
- Directories with > 10,000 entries
- Production filesystems
- Performance critical

**Implementation:**
```rust
impl Directory {
    pub fn lookup(&self, name: &str) -> Result<InodeId> {
        let mut block_id = self.root_block;

        loop {
            let node: BTreeNode = read_block(block_id)?;

            if node.is_leaf {
                // Linear search in leaf (or binary search)
                for (i, key) in node.keys.iter().enumerate() {
                    if key == name {
                        return Ok(node.values[i]);
                    }
                }
                return Err(FsError::NotFound(name.to_string()));
            } else {
                // Find child to descend into
                let child_index = node.keys.iter()
                    .position(|k| name < k)
                    .unwrap_or(node.num_keys);
                block_id = node.children[child_index];
            }
        }
    }
}
```

---

### Option 3: Hash-based Tree (ext4 HTree)

**Structure:**
```rust
pub struct HTreeNode {
    pub hash_table: [Option<BlockId>; 256],  // Hash buckets
}

pub struct Directory {
    pub htree_root: BlockId,
}
```

**Example:**
```
hash("a.txt") = 0x3A → Bucket 58 → Block 200
hash("b.txt") = 0x7F → Bucket 127 → Block 201

Block 200:
  ["a.txt" → Inode5, "aaa.txt" → Inode7]  // Collision handling

Block 201:
  ["b.txt" → Inode10]
```

**Pros:**
- ✅ O(1) average-case lookup
- ✅ Good for very large directories
- ✅ Simple implementation

**Cons:**
- ❌ No ordering (can't list sorted)
- ❌ Hash collisions
- ❌ Need to rehash on resize

**When to use:**
- Lookup performance critical
- Ordering not important
- Directories with random names

---

## K-ary Trees for Files

### Option 1: Direct + Indirect Blocks (Current Design) ✅

```rust
pub struct Inode {
    pub direct: [BlockId; 12],        // K=12
    pub indirect: BlockId,            // K=512
    pub double_indirect: BlockId,     // K=512
    pub triple_indirect: BlockId,     // K=512
}
```

**File size limits:**
- Direct: 12 × 4KB = **48 KB**
- + Indirect: 512 × 4KB = **2 MB**
- + Double: 512 × 512 × 4KB = **1 GB**
- + Triple: 512 × 512 × 512 × 4KB = **512 GB**

**Pros:**
- ✅ Simple for small files (most files < 48KB)
- ✅ No overhead for small files
- ✅ Well-understood (used by ext2/3/4, UFS, etc.)
- ✅ Educational - easy to explain

**Cons:**
- ❌ Inefficient for large files (many pointer lookups)
- ❌ Fragmentation possible
- ❌ Wasted space (all inodes have indirect pointers)

---

### Option 2: Extent-based (Modern Approach)

**Structure:**
```rust
pub struct Extent {
    pub start_block: BlockId,   // First block
    pub length: u32,            // Number of contiguous blocks
}

pub struct Inode {
    pub extents: Vec<Extent>,   // List of extents
}
```

**Example:**
```
File with 100 blocks, stored in 3 extents:
  Extent 0: blocks 1000-1049 (50 blocks)
  Extent 1: blocks 2000-2029 (30 blocks)
  Extent 2: blocks 3000-3019 (20 blocks)

Instead of 100 pointers, only 3 extents!
```

**Pros:**
- ✅ Efficient for large files
- ✅ Fewer pointers (less metadata)
- ✅ Better for contiguous allocation
- ✅ Used by modern filesystems (ext4, btrfs, XFS)

**Cons:**
- ❌ More complex allocation
- ❌ Need extent tree for many extents
- ❌ Fragmentation still possible

**When to use:**
- Large files common
- Sequential I/O important
- Modern filesystem

---

### Option 3: B-tree for Blocks (btrfs approach)

**Structure:**
```rust
pub struct FileBlockTree {
    pub root: BlockId,  // Root of B-tree mapping offset → block
}

pub struct BTreeNode {
    pub keys: Vec<u64>,        // File offsets
    pub values: Vec<BlockId>,  // Block addresses
}
```

**Example:**
```
File block tree (maps file offset → physical block):
  Offset 0-4095 → Block 1000
  Offset 4096-8191 → Block 2500
  Offset 8192-12287 → Block 3700
  ...
```

**Pros:**
- ✅ Supports very large files
- ✅ Efficient sparse files (holes)
- ✅ Can store inline data
- ✅ Most flexible

**Cons:**
- ❌ Complex implementation
- ❌ Overhead for small files
- ❌ More disk I/O

**When to use:**
- Copy-on-write filesystems
- Snapshots/clones
- Very large files
- Advanced features

---

## Real-World Examples

### ext2/ext3/ext4 (Classic)

**Directories:**
- **ext2/ext3**: Linear list (no K-ary tree)
  - Simple array of entries
  - O(n) lookup - slow for large directories!

- **ext4**: HTree (hash-based tree)
  - 2-level hash table
  - O(1) average lookup
  - Used for directories > 1000 entries

**Files:**
- Direct blocks: 12
- Indirect: 1 level (K=1024)
- Double: 2 levels
- Triple: 3 levels
- Max file: ~2TB

**Like PSLFS V2!**

---

### XFS (High Performance)

**Directories:**
- **Small**: Embedded in inode
- **Medium**: B-tree (K varies)
- **Large**: Hash table + B-tree hybrid

**Files:**
- **Extents** instead of blocks
- B+ tree for extent map
- Very efficient for large files

---

### btrfs (Modern)

**Everything is a B-tree!**
- Directories: B-tree (K=varies)
- Files: B-tree of extents
- Metadata: B-tree
- Free space: B-tree

**Pros:**
- Snapshots
- Copy-on-write
- Checksums

**Cons:**
- Complex!

---

## Recommendations for PSLFS V2

### Current Design (Good!) ✅

```rust
// Directories: BTreeMap
pub struct Directory {
    pub entries: BTreeMap<String, InodeId>,  // Good for educational
}

// Files: Direct + Indirect
pub struct Inode {
    pub direct: [BlockId; 12],
    pub indirect: BlockId,
    // ...
}
```

**Why this is right:**
- ✅ Simple to understand (educational goal)
- ✅ BTreeMap is K-ary (efficient)
- ✅ Direct blocks fast for small files (common case)
- ✅ Indirect blocks handle large files
- ✅ Well-documented pattern (ext2-like)

---

### Phase 1: Keep Current Design (Weeks 1-10)

**Focus on:**
- Getting it working
- Testing
- Documentation
- FUSE integration

**Don't optimize prematurely!**

---

### Phase 2: Advanced Features (Weeks 11+)

Add these as **optional learning modules**:

#### Module 1: On-Disk B-tree for Large Directories

```rust
// New directory implementation
pub enum DirectoryImpl {
    Small(BTreeMap<String, InodeId>),    // < 1000 entries
    Large(OnDiskBTree),                  // > 1000 entries
}
```

**Learning objectives:**
- Understand B-tree node splitting
- Learn about disk-based data structures
- Practice serialization

---

#### Module 2: Extent-based File Storage

```rust
pub struct Inode {
    pub storage: FileStorage,
}

pub enum FileStorage {
    Direct([BlockId; 12]),       // Small files
    Extents(Vec<Extent>),        // Large files
}

pub struct Extent {
    pub start: BlockId,
    pub length: u32,
}
```

**Learning objectives:**
- Understand extent allocation
- Learn about fragmentation
- Practice allocation strategies

---

#### Module 3: Performance Comparison

Implement all three approaches and benchmark:

```bash
# Benchmark script
pslfs benchmark --approach btreemap
pslfs benchmark --approach ondisk-btree
pslfs benchmark --approach extents

# Results:
# BTreeMap: 10ms per 1000 lookups
# OnDisk B-tree: 50ms per 1000 lookups (cold cache), 5ms (warm)
# Extents: 3ms per 1000 sequential reads
```

**Learning objectives:**
- Understand performance trade-offs
- Learn about caching effects
- Practice benchmarking

---

## Summary Table

| Approach | Complexity | Performance | Use Case |
|----------|-----------|-------------|----------|
| **BTreeMap** (current) | Low | O(log n) | Educational, < 10K entries |
| **On-disk B-tree** | High | O(log_K n) | Production, > 10K entries |
| **HTree** | Medium | O(1) avg | Large dirs, lookup-heavy |
| **Direct blocks** (current) | Low | O(1) | Small files (< 48KB) |
| **Indirect blocks** (current) | Low | O(k) | Medium files (< 512GB) |
| **Extents** | Medium | O(1) | Large sequential files |
| **B-tree blocks** | High | O(log n) | Very large files, sparse |

---

## Final Answer

**Q: Should PSLFS use K-ary trees?**

**A: It already does!**

- **Directories**: BTreeMap is a B-tree (K-ary tree)
- **Files**: Indirect blocks form a K-ary tree (K≈512)

**This is the right choice for V2 because:**

1. ✅ **Educational**: Easy to understand and implement
2. ✅ **Efficient**: O(log n) for most operations
3. ✅ **Standard library**: BTreeMap is well-tested
4. ✅ **Extensible**: Can add advanced K-ary trees later
5. ✅ **Real-world**: Similar to ext2/ext3

**For advanced users:**
- Add on-disk B-trees as learning module
- Compare performance with BTreeMap
- Understand production filesystem design

---

## Code Example: Comparing Approaches

```rust
// Current: BTreeMap (simple)
pub struct DirectoryV1 {
    pub entries: BTreeMap<String, InodeId>,
}

impl DirectoryV1 {
    pub fn lookup(&self, name: &str) -> Option<InodeId> {
        self.entries.get(name).copied()  // O(log n)
    }
}

// Advanced: On-disk B-tree (complex but scalable)
pub struct DirectoryV2 {
    pub root_block: BlockId,
}

impl DirectoryV2 {
    pub fn lookup(&self, name: &str) -> Result<InodeId> {
        let mut current = self.root_block;

        loop {
            let node: BTreeNode = read_block(current)?;

            if node.is_leaf {
                return node.find(name);
            } else {
                current = node.find_child(name);
            }
        }
    }
}

// Future: Choose based on size
pub enum Directory {
    Small(DirectoryV1),   // < 1000 entries
    Large(DirectoryV2),   // >= 1000 entries
}
```

---

**Conclusion:** The current design is excellent. K-ary trees (B-trees) are already there. Advanced versions can come later as optional learning modules!
