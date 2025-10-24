//! Core type definitions for PSLFS V2
//!
//! This module defines the fundamental types used throughout the filesystem,
//! including identifiers, blocks, and other core structures.

use std::fmt;

/// Unique identifier for a block in the filesystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId(pub u64);

impl BlockId {
    /// Create a new BlockId
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the underlying u64 value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockId({})", self.0)
    }
}

/// Unique identifier for an inode (file or directory)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InodeId(pub u64);

impl InodeId {
    /// Create a new InodeId
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the underlying u64 value
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Root inode ID (always 1)
    pub fn root() -> Self {
        Self(1)
    }
}

impl fmt::Display for InodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InodeId({})", self.0)
    }
}

/// Unique identifier for a user
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(pub u32);

impl UserId {
    /// Create a new UserId
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the underlying u32 value
    pub fn value(&self) -> u32 {
        self.0
    }

    /// Root user ID (always 0)
    pub fn root() -> Self {
        Self(0)
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UserId({})", self.0)
    }
}

/// Unique identifier for a user session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(pub u64);

impl SessionId {
    /// Create a new SessionId
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the underlying u64 value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SessionId({})", self.0)
    }
}

/// Type of inode (file or directory)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InodeKind {
    File,
    Directory,
    // Symlink(PathBuf), // Future extension
}

impl fmt::Display for InodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InodeKind::File => write!(f, "File"),
            InodeKind::Directory => write!(f, "Directory"),
        }
    }
}

/// Filesystem state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsState {
    Clean,
    Dirty,
    Error,
}

impl fmt::Display for FsState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsState::Clean => write!(f, "Clean"),
            FsState::Dirty => write!(f, "Dirty"),
            FsState::Error => write!(f, "Error"),
        }
    }
}

/// A block of data (4KB)
#[derive(Debug, Clone)]
pub struct Block {
    pub data: Vec<u8>,
}

impl Block {
    /// Block size in bytes (4KB)
    pub const SIZE: usize = 4096;

    /// Create a new block filled with zeros
    pub fn new() -> Self {
        Self {
            data: vec![0; Self::SIZE],
        }
    }

    /// Create a block from a slice (truncated or padded to 4KB)
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut data = vec![0; Self::SIZE];
        let copy_len = std::cmp::min(slice.len(), Self::SIZE);
        data[..copy_len].copy_from_slice(&slice[..copy_len]);
        Self { data }
    }

    /// Get the data as a slice
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get the data as a mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_id() {
        let id = BlockId::new(42);
        assert_eq!(id.value(), 42);
        assert_eq!(id, BlockId(42));
    }

    #[test]
    fn test_inode_id() {
        let id = InodeId::new(1);
        assert_eq!(id.value(), 1);
        assert_eq!(InodeId::root(), InodeId(1));
    }

    #[test]
    fn test_user_id() {
        let id = UserId::new(1000);
        assert_eq!(id.value(), 1000);
        assert_eq!(UserId::root(), UserId(0));
    }

    #[test]
    fn test_block_operations() {
        let block = Block::new();
        assert_eq!(block.data.len(), Block::SIZE);
        assert!(block.data.iter().all(|&b| b == 0));

        let data = b"Hello, World!";
        let block2 = Block::from_slice(data);
        assert_eq!(&block2.data[..data.len()], data);

        // Check that remaining bytes are zero
        for i in data.len()..Block::SIZE {
            assert_eq!(block2.data[i], 0);
        }
    }

    #[test]
    fn test_inode_kind_display() {
        assert_eq!(format!("{}", InodeKind::File), "File");
        assert_eq!(format!("{}", InodeKind::Directory), "Directory");
    }
}