//! Inode implementation for PSLFS V2
//!
//! Inodes represent files and directories in the filesystem. Each inode contains
//! metadata and pointers to data blocks. This module implements the core inode
//! functionality with direct block pointers (no indirect blocks initially).

use crate::{BlockId, InodeId, UserId, InodeKind, Result};
use std::time::{SystemTime, UNIX_EPOCH};

/// Errors that can occur during inode operations
#[derive(Debug, thiserror::Error)]
pub enum InodeError {
    #[error("Block index {0} out of range")]
    InvalidBlockIndex(usize),

    #[error("Too many blocks for inode")]
    TooManyBlocks,
}

pub type InodeResult<T> = std::result::Result<T, InodeError>;

/// Inode representing a file or directory
#[derive(Debug, Clone)]
pub struct Inode {
    /// Unique inode identifier
    pub id: InodeId,

    /// Type of inode (file or directory)
    pub kind: InodeKind,

    /// File size in bytes
    pub size: u64,

    /// Number of hard links to this inode
    pub nlink: u32,

    /// Direct block pointers (simplified - no indirect blocks)
    pub blocks: Vec<BlockId>,

    /// File mode (permissions)
    pub mode: u16,

    /// Owner user ID
    pub uid: UserId,

    /// Group ID
    pub gid: u32,

    /// Creation time
    pub created: SystemTime,

    /// Last modification time
    pub modified: SystemTime,

    /// Last access time
    pub accessed: SystemTime,
}

impl Inode {
    /// Maximum number of direct blocks per inode
    /// With 4KB blocks, this gives max file size of ~16MB
    pub const MAX_BLOCKS: usize = 4096;

    /// Create a new inode
    ///
    /// # Arguments
    /// * `id` - Unique inode identifier
    /// * `kind` - Type of inode (file or directory)
    /// * `mode` - Permission mode (e.g., 0o644 for files, 0o755 for directories)
    /// * `uid` - Owner user ID
    /// * `gid` - Group ID
    pub fn new(id: InodeId, kind: InodeKind, mode: u16, uid: UserId, gid: u32) -> Self {
        let now = SystemTime::now();

        // Directories start with nlink=2 (. and ..)
        let nlink = match kind {
            InodeKind::Directory => 2,
            InodeKind::File => 1,
        };

        Self {
            id,
            kind,
            size: 0,
            nlink,
            blocks: Vec::new(),
            mode,
            uid,
            gid,
            created: now,
            modified: now,
            accessed: now,
        }
    }

    /// Create a new file inode
    pub fn new_file(id: InodeId, mode: u16, uid: UserId, gid: u32) -> Self {
        Self::new(id, InodeKind::File, mode, uid, gid)
    }

    /// Create a new directory inode
    pub fn new_directory(id: InodeId, mode: u16, uid: UserId, gid: u32) -> Self {
        Self::new(id, InodeKind::Directory, mode, uid, gid)
    }

    /// Set a block pointer at the given index
    ///
    /// # Arguments
    /// * `index` - Block index (0-based)
    /// * `block_id` - Block identifier to set
    pub fn set_block(&mut self, index: usize, block_id: BlockId) -> InodeResult<()> {
        if index >= Self::MAX_BLOCKS {
            return Err(InodeError::InvalidBlockIndex(index));
        }

        // Extend blocks vector if necessary
        if index >= self.blocks.len() {
            self.blocks.resize(index + 1, BlockId::new(0));
        }

        self.blocks[index] = block_id;
        self.update_size();
        self.modified = SystemTime::now();

        Ok(())
    }

    /// Get a block pointer at the given index
    ///
    /// # Arguments
    /// * `index` - Block index (0-based)
    pub fn get_block(&self, index: usize) -> Option<BlockId> {
        if index < self.blocks.len() {
            let block_id = self.blocks[index];
            if block_id.value() != 0 {
                Some(block_id)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Add a block to the end of the block list
    pub fn add_block(&mut self, block_id: BlockId) -> InodeResult<()> {
        if self.blocks.len() >= Self::MAX_BLOCKS {
            return Err(InodeError::TooManyBlocks);
        }

        self.blocks.push(block_id);
        self.update_size();
        self.modified = SystemTime::now();

        Ok(())
    }

    /// Remove the last block from the block list
    pub fn remove_last_block(&mut self) -> Option<BlockId> {
        let block = self.blocks.pop();
        self.update_size();
        self.modified = SystemTime::now();
        block
    }

    /// Get the number of blocks used by this inode
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Update the file size based on block count
    fn update_size(&mut self) {
        self.size = self.blocks.len() as u64 * 4096;
    }

    /// Update access time
    pub fn touch_accessed(&mut self) {
        self.accessed = SystemTime::now();
    }

    /// Update modification time
    pub fn touch_modified(&mut self) {
        self.modified = SystemTime::now();
    }

    /// Get inode metadata for filesystem operations
    pub fn metadata(&self) -> InodeMetadata {
        InodeMetadata {
            inode_id: self.id,
            kind: self.kind,
            size: self.size,
            mode: self.mode,
            uid: self.uid,
            gid: self.gid,
            nlink: self.nlink,
            created: self.created,
            modified: self.modified,
            accessed: self.accessed,
            block_count: self.blocks.len(),
        }
    }

    /// Check if this inode represents a directory
    pub fn is_directory(&self) -> bool {
        matches!(self.kind, InodeKind::Directory)
    }

    /// Check if this inode represents a file
    pub fn is_file(&self) -> bool {
        matches!(self.kind, InodeKind::File)
    }
}

/// Metadata about an inode (for filesystem API)
#[derive(Debug, Clone)]
pub struct InodeMetadata {
    pub inode_id: InodeId,
    pub kind: InodeKind,
    pub size: u64,
    pub mode: u16,
    pub uid: UserId,
    pub gid: u32,
    pub nlink: u32,
    pub created: SystemTime,
    pub modified: SystemTime,
    pub accessed: SystemTime,
    pub block_count: usize,
}

impl InodeMetadata {
    /// Get the file type character (like `ls -l`)
    pub fn file_type_char(&self) -> char {
        match self.kind {
            InodeKind::File => '-',
            InodeKind::Directory => 'd',
        }
    }

    /// Format permissions as a string (like `ls -l`)
    pub fn permissions_string(&self) -> String {
        format!("{}{}{}{}{}{}{}{}{}{}",
            self.file_type_char(),
            if self.mode & 0o400 != 0 { "r" } else { "-" },
            if self.mode & 0o200 != 0 { "w" } else { "-" },
            if self.mode & 0o100 != 0 { "x" } else { "-" },
            if self.mode & 0o040 != 0 { "r" } else { "-" },
            if self.mode & 0o020 != 0 { "w" } else { "-" },
            if self.mode & 0o010 != 0 { "x" } else { "-" },
            if self.mode & 0o004 != 0 { "r" } else { "-" },
            if self.mode & 0o002 != 0 { "w" } else { "-" },
            if self.mode & 0o001 != 0 { "x" } else { "-" },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inode_new_file() {
        let inode = Inode::new_file(InodeId::new(1), 0o644, UserId::new(1000), 1000);

        assert_eq!(inode.id, InodeId(1));
        assert_eq!(inode.kind, InodeKind::File);
        assert_eq!(inode.mode, 0o644);
        assert_eq!(inode.uid, UserId(1000));
        assert_eq!(inode.gid, 1000);
        assert_eq!(inode.nlink, 1);
        assert_eq!(inode.size, 0);
        assert!(inode.blocks.is_empty());
    }

    #[test]
    fn test_inode_new_directory() {
        let inode = Inode::new_directory(InodeId::new(2), 0o755, UserId::new(1000), 1000);

        assert_eq!(inode.id, InodeId(2));
        assert_eq!(inode.kind, InodeKind::Directory);
        assert_eq!(inode.mode, 0o755);
        assert_eq!(inode.nlink, 2); // . and ..
        assert_eq!(inode.size, 0);
    }

    #[test]
    fn test_inode_block_operations() {
        let mut inode = Inode::new_file(InodeId::new(1), 0o644, UserId::new(1000), 1000);

        // Add blocks
        inode.add_block(BlockId::new(100)).unwrap();
        inode.add_block(BlockId::new(101)).unwrap();

        assert_eq!(inode.block_count(), 2);
        assert_eq!(inode.get_block(0), Some(BlockId(100)));
        assert_eq!(inode.get_block(1), Some(BlockId(101)));
        assert_eq!(inode.get_block(2), None);

        // Set block at specific index
        inode.set_block(5, BlockId::new(200)).unwrap();
        assert_eq!(inode.get_block(5), Some(BlockId(200)));
        assert_eq!(inode.block_count(), 6); // Extended to index 5

        // Remove last block
        let removed = inode.remove_last_block();
        assert_eq!(removed, Some(BlockId(200)));
        assert_eq!(inode.block_count(), 5);
    }

    #[test]
    fn test_inode_block_limits() {
        let mut inode = Inode::new_file(InodeId::new(1), 0o644, UserId::new(1000), 1000);

        // Should fail: too many blocks
        for i in 0..Inode::MAX_BLOCKS {
            inode.add_block(BlockId::new(i as u64)).unwrap();
        }

        assert!(matches!(inode.add_block(BlockId::new(9999)), Err(InodeError::TooManyBlocks)));

        // Should fail: invalid block index
        assert!(matches!(inode.set_block(Inode::MAX_BLOCKS, BlockId::new(1)), Err(InodeError::InvalidBlockIndex(_))));
    }

    #[test]
    fn test_inode_metadata() {
        let inode = Inode::new_file(InodeId::new(1), 0o644, UserId::new(1000), 1000);
        let metadata = inode.metadata();

        assert_eq!(metadata.inode_id, InodeId(1));
        assert_eq!(metadata.kind, InodeKind::File);
        assert_eq!(metadata.mode, 0o644);
        assert_eq!(metadata.uid, UserId(1000));
        assert_eq!(metadata.size, 0);
        assert_eq!(metadata.block_count, 0);
    }

    #[test]
    fn test_inode_type_checks() {
        let file_inode = Inode::new_file(InodeId::new(1), 0o644, UserId::new(1000), 1000);
        let dir_inode = Inode::new_directory(InodeId::new(2), 0o755, UserId::new(1000), 1000);

        assert!(file_inode.is_file());
        assert!(!file_inode.is_directory());

        assert!(dir_inode.is_directory());
        assert!(!dir_inode.is_file());
    }

    #[test]
    fn test_permissions_string() {
        let mut inode = Inode::new_file(InodeId::new(1), 0o644, UserId::new(1000), 1000);
        let metadata = inode.metadata();

        assert_eq!(metadata.permissions_string(), "-rw-r--r--");

        inode.mode = 0o755;
        let metadata = inode.metadata();
        assert_eq!(metadata.permissions_string(), "-rwxr-xr-x");
    }
}