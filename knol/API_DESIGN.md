# PSLFS V2 - API Design

**Version:** 2.0
**Status:** Design Phase
**Last Updated:** 2025-10-22

---

## Table of Contents

1. [Overview](#overview)
2. [Public Library API](#public-library-api)
3. [CLI Interface](#cli-interface)
4. [FUSE Interface](#fuse-interface)
5. [Error Handling](#error-handling)
6. [Usage Examples](#usage-examples)

---

## Overview

PSLFS V2 provides three interfaces:

1. **Library API** (`lib.rs`) - Rust library for embedding in applications
2. **CLI Tools** (`main.rs`) - Command-line utilities
3. **FUSE Driver** (`fuse/mod.rs`) - Mount as real filesystem

**Design Principles:**
- Type-safe APIs (use newtypes, not raw integers)
- Explicit error handling (no panics in library code)
- Async-ready (return `Result`, not futures - can wrap later)
- Well-documented (rustdoc with examples)

---

## Public Library API

### Core Filesystem Operations

```rust
use pslfs::{Filesystem, InodeId, BlockId, Result};
use std::path::Path;

/// Main filesystem handle
pub struct Filesystem {
    // Private fields
}

impl Filesystem {
    /// Create a new filesystem at the given path
    ///
    /// # Arguments
    /// * `path` - Path where filesystem files will be stored
    /// * `size` - Total size in bytes
    /// * `label` - Optional volume label
    ///
    /// # Errors
    /// Returns error if path exists or size is invalid
    ///
    /// # Example
    /// ```
    /// use pslfs::Filesystem;
    ///
    /// let fs = Filesystem::create(
    ///     "/tmp/myfs",
    ///     10 * 1024 * 1024,  // 10MB
    ///     Some("MyFS")
    /// )?;
    /// ```
    pub fn create<P: AsRef<Path>>(
        path: P,
        size: u64,
        label: Option<&str>,
    ) -> Result<Self>;

    /// Open an existing filesystem
    ///
    /// # Arguments
    /// * `path` - Path to filesystem files
    /// * `read_only` - Open in read-only mode
    ///
    /// # Example
    /// ```
    /// use pslfs::Filesystem;
    ///
    /// let fs = Filesystem::open("/tmp/myfs", false)?;
    /// ```
    pub fn open<P: AsRef<Path>>(path: P, read_only: bool) -> Result<Self>;

    /// Close filesystem and flush all pending writes
    pub fn close(self) -> Result<()>;

    /// Sync all dirty data to disk
    pub fn sync(&mut self) -> Result<()>;

    /// Get filesystem statistics
    pub fn stats(&self) -> FsStats;

    /// Run filesystem check and repair
    pub fn fsck(&mut self, repair: bool) -> Result<FsckReport>;
}
```

---

### File Operations

```rust
impl Filesystem {
    /// Create a new file
    ///
    /// # Arguments
    /// * `parent` - Parent directory inode
    /// * `name` - Filename
    /// * `mode` - Permission mode (e.g., 0o644)
    ///
    /// # Returns
    /// Inode ID of created file
    ///
    /// # Errors
    /// - `AlreadyExists` if file with same name exists
    /// - `PermissionDenied` if no write permission on parent
    /// - `NotADirectory` if parent is not a directory
    pub fn create_file(
        &mut self,
        parent: InodeId,
        name: &str,
        mode: u16,
    ) -> Result<InodeId>;

    /// Read from a file
    ///
    /// # Arguments
    /// * `inode` - File inode
    /// * `offset` - Byte offset to start reading
    /// * `buf` - Buffer to read into
    ///
    /// # Returns
    /// Number of bytes read
    ///
    /// # Errors
    /// - `NotAFile` if inode is not a file
    /// - `PermissionDenied` if no read permission
    pub fn read_file(
        &self,
        inode: InodeId,
        offset: u64,
        buf: &mut [u8],
    ) -> Result<usize>;

    /// Write to a file
    ///
    /// # Arguments
    /// * `inode` - File inode
    /// * `offset` - Byte offset to start writing
    /// * `data` - Data to write
    ///
    /// # Returns
    /// Number of bytes written
    ///
    /// # Errors
    /// - `NotAFile` if inode is not a file
    /// - `PermissionDenied` if no write permission
    /// - `NoSpace` if filesystem is full
    pub fn write_file(
        &mut self,
        inode: InodeId,
        offset: u64,
        data: &[u8],
    ) -> Result<usize>;

    /// Truncate file to given size
    pub fn truncate(&mut self, inode: InodeId, size: u64) -> Result<()>;

    /// Delete a file
    ///
    /// # Arguments
    /// * `parent` - Parent directory inode
    /// * `name` - Filename to delete
    ///
    /// # Errors
    /// - `NotFound` if file doesn't exist
    /// - `PermissionDenied` if no write permission
    pub fn delete_file(&mut self, parent: InodeId, name: &str) -> Result<()>;
}
```

---

### Directory Operations

```rust
impl Filesystem {
    /// Create a new directory
    ///
    /// # Arguments
    /// * `parent` - Parent directory inode
    /// * `name` - Directory name
    /// * `mode` - Permission mode (e.g., 0o755)
    ///
    /// # Returns
    /// Inode ID of created directory
    pub fn create_dir(
        &mut self,
        parent: InodeId,
        name: &str,
        mode: u16,
    ) -> Result<InodeId>;

    /// Read directory entries
    ///
    /// # Arguments
    /// * `inode` - Directory inode
    ///
    /// # Returns
    /// Vector of directory entries
    ///
    /// # Errors
    /// - `NotADirectory` if inode is not a directory
    /// - `PermissionDenied` if no read permission
    pub fn read_dir(&self, inode: InodeId) -> Result<Vec<DirEntry>>;

    /// Lookup entry in directory
    ///
    /// # Arguments
    /// * `parent` - Directory inode
    /// * `name` - Entry name to lookup
    ///
    /// # Returns
    /// Inode ID of entry if found
    pub fn lookup(&self, parent: InodeId, name: &str) -> Result<InodeId>;

    /// Delete a directory
    ///
    /// # Arguments
    /// * `parent` - Parent directory inode
    /// * `name` - Directory name to delete
    ///
    /// # Errors
    /// - `NotFound` if directory doesn't exist
    /// - `DirectoryNotEmpty` if directory has entries
    /// - `PermissionDenied` if no write permission
    pub fn delete_dir(&mut self, parent: InodeId, name: &str) -> Result<()>;

    /// Get root directory inode
    pub fn root(&self) -> InodeId;
}
```

---

### Path-Based Operations

```rust
impl Filesystem {
    /// Resolve a path to an inode
    ///
    /// # Arguments
    /// * `path` - Absolute or relative path
    /// * `cwd` - Current working directory (for relative paths)
    ///
    /// # Example
    /// ```
    /// let inode = fs.path_to_inode("/home/user/file.txt", None)?;
    /// ```
    pub fn path_to_inode(
        &self,
        path: &str,
        cwd: Option<InodeId>,
    ) -> Result<InodeId>;

    /// Read entire file by path (convenience function)
    pub fn read_to_string(&self, path: &str) -> Result<String>;

    /// Write entire file by path (convenience function)
    pub fn write_all(&mut self, path: &str, data: &[u8]) -> Result<()>;
}
```

---

### Metadata Operations

```rust
/// File/directory metadata
#[derive(Debug, Clone)]
pub struct Metadata {
    pub inode: InodeId,
    pub kind: InodeKind,
    pub size: u64,
    pub mode: u16,
    pub uid: u32,
    pub gid: u32,
    pub nlink: u32,
    pub created: SystemTime,
    pub modified: SystemTime,
    pub accessed: SystemTime,
}

impl Filesystem {
    /// Get metadata for an inode
    pub fn metadata(&self, inode: InodeId) -> Result<Metadata>;

    /// Change permissions
    pub fn chmod(&mut self, inode: InodeId, mode: u16) -> Result<()>;

    /// Change owner
    pub fn chown(&mut self, inode: InodeId, uid: u32, gid: u32) -> Result<()>;

    /// Update access/modification time
    pub fn touch(&mut self, inode: InodeId) -> Result<()>;
}
```

---

### User Management

```rust
use pslfs::{UserId, User};

impl Filesystem {
    /// Create a new user
    ///
    /// # Arguments
    /// * `username` - Username (must be unique)
    /// * `password` - Plaintext password (will be hashed)
    ///
    /// # Returns
    /// User ID of created user
    pub fn create_user(&mut self, username: &str, password: &str) -> Result<UserId>;

    /// Authenticate user
    ///
    /// # Arguments
    /// * `username` - Username
    /// * `password` - Password to verify
    ///
    /// # Returns
    /// User ID if authentication succeeds
    pub fn authenticate(&self, username: &str, password: &str) -> Result<UserId>;

    /// Get user information
    pub fn get_user(&self, id: UserId) -> Result<User>;

    /// List all users
    pub fn list_users(&self) -> Result<Vec<User>>;

    /// Delete user
    pub fn delete_user(&mut self, id: UserId) -> Result<()>;
}
```

---

### Statistics and Debugging

```rust
/// Filesystem statistics
#[derive(Debug, Clone)]
pub struct FsStats {
    pub total_blocks: u64,
    pub free_blocks: u64,
    pub total_inodes: u64,
    pub free_inodes: u64,
    pub block_size: u32,
    pub mount_count: u32,
    pub state: FsState,
}

/// Filesystem check report
#[derive(Debug)]
pub struct FsckReport {
    pub errors: Vec<FsckError>,
    pub warnings: Vec<String>,
    pub repaired: bool,
}

impl Filesystem {
    /// Get detailed statistics
    pub fn stats(&self) -> FsStats;

    /// Run filesystem check
    pub fn fsck(&mut self, repair: bool) -> Result<FsckReport>;

    /// Dump inode information (for debugging)
    pub fn dump_inode(&self, inode: InodeId) -> Result<String>;

    /// Dump entire filesystem structure (for debugging)
    pub fn dump_tree(&self) -> Result<String>;
}
```

---

## CLI Interface

### Commands

```bash
# pslfs - PSLFS V2 command-line utility
#
# USAGE:
#     pslfs <COMMAND> [OPTIONS]
#
# COMMANDS:
#     init        Create a new filesystem
#     mount       Mount filesystem via FUSE
#     umount      Unmount FUSE filesystem
#     ls          List directory contents
#     cat         Display file contents
#     cp          Copy file into/out of filesystem
#     rm          Remove file or directory
#     mkdir       Create directory
#     stat        Display file/directory metadata
#     check       Check filesystem consistency
#     stats       Show filesystem statistics
#     user        User management commands
#     help        Print this message or the help of the given subcommand(s)
```

---

### Command Details

#### `pslfs init`

```bash
# Create a new filesystem
pslfs init [OPTIONS] <PATH>

OPTIONS:
    -s, --size <SIZE>        Size in bytes (supports K, M, G suffix)
    -l, --label <LABEL>      Volume label
    -u, --user <USER>        Initial username
    -p, --password <PASS>    Initial password (prompts if not provided)

EXAMPLES:
    # Create 100MB filesystem
    pslfs init /tmp/myfs --size 100M --label "My Filesystem"

    # Create with initial user
    pslfs init /tmp/myfs --size 1G --user admin
```

#### `pslfs mount`

```bash
# Mount filesystem via FUSE
pslfs mount [OPTIONS] <FS_PATH> <MOUNT_POINT>

OPTIONS:
    -r, --read-only          Mount read-only
    -d, --debug              Enable debug output
    -f, --foreground         Run in foreground
    -u, --user <USER>        Username for authentication
    -p, --password <PASS>    Password (prompts if not provided)

EXAMPLES:
    # Mount filesystem
    pslfs mount /tmp/myfs /mnt/pslfs --user admin

    # Mount read-only with debug
    pslfs mount /tmp/myfs /mnt/pslfs -r -d -f
```

#### `pslfs ls`

```bash
# List directory contents (direct access, no FUSE required)
pslfs ls [OPTIONS] <FS_PATH>:<DIR_PATH>

OPTIONS:
    -l, --long               Long format with metadata
    -a, --all                Show hidden files
    -h, --human-readable     Human-readable sizes

EXAMPLES:
    # List root directory
    pslfs ls /tmp/myfs:/

    # Long format
    pslfs ls -l /tmp/myfs:/home/user
```

#### `pslfs cat`

```bash
# Display file contents (direct access)
pslfs cat <FS_PATH>:<FILE_PATH>

EXAMPLES:
    pslfs cat /tmp/myfs:/home/user/readme.txt
```

#### `pslfs cp`

```bash
# Copy files into/out of filesystem
pslfs cp [OPTIONS] <SOURCE> <DEST>

EXAMPLES:
    # Copy file into filesystem
    pslfs cp /tmp/local.txt /tmp/myfs:/remote.txt

    # Copy file out of filesystem
    pslfs cp /tmp/myfs:/remote.txt /tmp/local.txt
```

#### `pslfs check`

```bash
# Check filesystem consistency
pslfs check [OPTIONS] <FS_PATH>

OPTIONS:
    -r, --repair             Attempt to repair errors
    -v, --verbose            Verbose output

EXAMPLES:
    # Check filesystem
    pslfs check /tmp/myfs

    # Check and repair
    pslfs check /tmp/myfs --repair
```

#### `pslfs stats`

```bash
# Show filesystem statistics
pslfs stats <FS_PATH>

OPTIONS:
    -j, --json               Output as JSON

EXAMPLES:
    # Show stats
    pslfs stats /tmp/myfs

    # JSON output
    pslfs stats /tmp/myfs --json
```

#### `pslfs user`

```bash
# User management
pslfs user <SUBCOMMAND>

SUBCOMMANDS:
    add         Add a new user
    list        List all users
    remove      Remove a user
    passwd      Change user password

EXAMPLES:
    # Add user
    pslfs user add /tmp/myfs --username alice

    # List users
    pslfs user list /tmp/myfs

    # Change password
    pslfs user passwd /tmp/myfs --username alice
```

---

## FUSE Interface

The FUSE interface implements standard filesystem operations:

```rust
use fuser::{Filesystem, Request, ReplyEntry, ReplyAttr, ReplyData, ReplyWrite};

impl Filesystem for PslfsFuse {
    /// Look up a directory entry by name
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry);

    /// Get file attributes
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr);

    /// Set file attributes
    fn setattr(&mut self, _req: &Request, ino: u64, mode: Option<u32>, ...);

    /// Read directory
    fn readdir(&mut self, _req: &Request, ino: u64, fh: u64, offset: i64, reply: ReplyDirectory);

    /// Create file
    fn create(&mut self, _req: &Request, parent: u64, name: &OsStr, mode: u32, ...);

    /// Open file
    fn open(&mut self, _req: &Request, ino: u64, flags: i32, reply: ReplyOpen);

    /// Read data
    fn read(&mut self, _req: &Request, ino: u64, fh: u64, offset: i64, size: u32, reply: ReplyData);

    /// Write data
    fn write(&mut self, _req: &Request, ino: u64, fh: u64, offset: i64, data: &[u8], reply: ReplyWrite);

    /// Create directory
    fn mkdir(&mut self, _req: &Request, parent: u64, name: &OsStr, mode: u32, reply: ReplyEntry);

    /// Remove file
    fn unlink(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty);

    /// Remove directory
    fn rmdir(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty);
}
```

---

## Error Handling

### Error Type

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Not a directory: inode {0}")]
    NotADirectory(InodeId),

    #[error("Not a file: inode {0}")]
    NotAFile(InodeId),

    #[error("Directory not empty: inode {0}")]
    DirectoryNotEmpty(InodeId),

    #[error("No space left on device")]
    NoSpace,

    #[error("Filesystem corrupted: {0}")]
    Corrupted(String),

    #[error("Invalid filename: {0}")]
    InvalidFilename(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Invalid block: {0}")]
    InvalidBlock(usize),

    #[error("Double free: block {0}")]
    DoubleFree(usize),
}

pub type Result<T> = std::result::Result<T, FsError>;
```

### Error Conversion for FUSE

```rust
impl From<FsError> for libc::c_int {
    fn from(err: FsError) -> Self {
        match err {
            FsError::NotFound(_) => libc::ENOENT,
            FsError::AlreadyExists(_) => libc::EEXIST,
            FsError::PermissionDenied => libc::EACCES,
            FsError::NotADirectory(_) => libc::ENOTDIR,
            FsError::NotAFile(_) => libc::EISDIR,
            FsError::DirectoryNotEmpty(_) => libc::ENOTEMPTY,
            FsError::NoSpace => libc::ENOSPC,
            FsError::Corrupted(_) => libc::EIO,
            FsError::InvalidFilename(_) => libc::EINVAL,
            FsError::InvalidPath(_) => libc::EINVAL,
            _ => libc::EIO,
        }
    }
}
```

---

## Usage Examples

### Example 1: Create and Use Filesystem (Library)

```rust
use pslfs::{Filesystem, InodeKind};

fn main() -> pslfs::Result<()> {
    // Create 10MB filesystem
    let mut fs = Filesystem::create(
        "/tmp/myfs",
        10 * 1024 * 1024,
        Some("Example FS"),
    )?;

    // Create user
    let uid = fs.create_user("alice", "password123")?;
    println!("Created user with ID: {:?}", uid);

    // Create directory
    let root = fs.root();
    let docs = fs.create_dir(root, "documents", 0o755)?;

    // Create file
    let file = fs.create_file(docs, "readme.txt", 0o644)?;

    // Write to file
    let data = b"Hello, PSLFS V2!";
    fs.write_file(file, 0, data)?;

    // Read back
    let mut buf = vec![0u8; 100];
    let n = fs.read_file(file, 0, &mut buf)?;
    println!("Read {} bytes: {}", n, String::from_utf8_lossy(&buf[..n]));

    // Get stats
    let stats = fs.stats();
    println!("Free blocks: {}/{}", stats.free_blocks, stats.total_blocks);

    // Close and flush
    fs.close()?;

    Ok(())
}
```

### Example 2: CLI Usage

```bash
#!/bin/bash

# Create filesystem
pslfs init /tmp/myfs --size 100M --label "TestFS" --user admin

# Mount it
mkdir -p /mnt/pslfs
pslfs mount /tmp/myfs /mnt/pslfs --user admin

# Use it like a normal filesystem
echo "Hello World" > /mnt/pslfs/hello.txt
cat /mnt/pslfs/hello.txt

mkdir /mnt/pslfs/documents
cp *.txt /mnt/pslfs/documents/

# Check stats
pslfs stats /tmp/myfs

# Unmount
pslfs umount /mnt/pslfs

# Direct access (no mount needed)
pslfs ls /tmp/myfs:/
pslfs cat /tmp/myfs:/hello.txt

# Check filesystem
pslfs check /tmp/myfs
```

### Example 3: FUSE in Code

```rust
use pslfs::fuse::PslfsFuse;
use fuser::MountOption;

fn main() -> pslfs::Result<()> {
    // Open filesystem
    let fs = pslfs::Filesystem::open("/tmp/myfs", false)?;

    // Create FUSE filesystem
    let fuse_fs = PslfsFuse::new(fs);

    // Mount options
    let options = vec![
        MountOption::RW,
        MountOption::FSName("pslfs".to_string()),
    ];

    // Mount (blocks until unmount)
    fuser::mount2(fuse_fs, "/mnt/pslfs", &options)?;

    Ok(())
}
```

---

## API Stability

### Stability Guarantees

- **Stable API** (v2.x): Public functions won't change signature
- **Disk Format**: V2 format is stable, will be readable by future versions
- **Error Types**: May add new variants, but won't remove existing ones

### Versioning Strategy

```
2.0.0 - Initial release
2.1.0 - Add new features (backwards compatible)
2.1.1 - Bug fixes
3.0.0 - Breaking changes (disk format or API)
```

---

This API design provides:
- ✅ Type-safe operations
- ✅ Clear error handling
- ✅ Multiple access methods (library, CLI, FUSE)
- ✅ Well-documented with examples
- ✅ Easy to learn and use
- ✅ Room for future extensions
