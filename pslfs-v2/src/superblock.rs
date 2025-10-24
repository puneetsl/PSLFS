//! # Superblock for PSLFS V2
//!
//! The superblock contains filesystem metadata and is always stored at block 0.
//! It provides essential information for mounting and managing the filesystem.

use crate::{Block, BlockId, InodeId, FsState, storage::{StorageBackend, StorageError, Result as StorageResult}};
use bincode::{deserialize, serialize};

/// Magic number for PSLFS V2 (0x50534C46535632 in little-endian)
const MAGIC: u64 = 0x50534C46535632;

/// Current filesystem version
const VERSION: u32 = 1;

/// Superblock structure containing filesystem metadata
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Superblock {
    /// Magic number for filesystem identification
    pub magic: u64,
    /// Filesystem version
    pub version: u32,
    /// Total number of blocks in the filesystem
    pub total_blocks: u64,
    /// Number of free blocks
    pub free_blocks: u64,
    /// Root inode ID
    pub root_inode: InodeId,
    /// Next free inode ID
    pub next_inode: u64,
    /// Creation time (Unix timestamp)
    pub created_at: u64,
    /// Last mount time
    pub mounted_at: u64,
    /// Mount count
    pub mount_count: u32,
    /// Filesystem state
    pub state: FsState,
    /// Checksum for integrity (simple sum for now)
    pub checksum: u64,
}

impl Superblock {
    /// Create a new superblock
    ///
    /// # Arguments
    /// * `total_blocks` - Total number of blocks in the filesystem
    /// * `root_inode` - Root inode ID (usually 1)
    pub fn new(total_blocks: u64, root_inode: InodeId) -> Self {
        let mut sb = Self {
            magic: MAGIC,
            version: VERSION,
            total_blocks,
            free_blocks: total_blocks - 1, // Reserve block 0 for superblock
            root_inode,
            next_inode: root_inode.value() + 1,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            mounted_at: 0,
            mount_count: 0,
            state: FsState::Clean,
            checksum: 0,
        };
        sb.update_checksum();
        sb
    }

    /// Serialize the superblock to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        serialize(self).expect("Failed to serialize superblock")
    }

    /// Deserialize superblock from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        deserialize(bytes)
    }

    /// Write superblock to a block
    pub fn to_block(&self) -> Block {
        let data = self.to_bytes();
        let mut block = Block::new();
        block.as_mut_slice()[..data.len()].copy_from_slice(&data);
        block
    }

    /// Read superblock from a block
    pub fn from_block(block: &Block) -> Result<Self, bincode::Error> {
        Self::from_bytes(block.as_slice())
    }

    /// Validate the superblock magic number and version
    pub fn is_valid(&self) -> bool {
        self.magic == MAGIC && self.version == VERSION
    }

    /// Update the checksum
    pub fn update_checksum(&mut self) {
        // Simple checksum: sum of all fields except checksum itself
        let state_value = match self.state {
            FsState::Clean => 0,
            FsState::Dirty => 1,
            FsState::Error => 2,
        };
        let checksum = self.magic
            .wrapping_add(self.version as u64)
            .wrapping_add(self.total_blocks)
            .wrapping_add(self.free_blocks)
            .wrapping_add(self.root_inode.value())
            .wrapping_add(self.next_inode)
            .wrapping_add(self.created_at)
            .wrapping_add(self.mounted_at)
            .wrapping_add(self.mount_count as u64)
            .wrapping_add(state_value);
        self.checksum = checksum;
    }

    /// Verify the checksum
    pub fn verify_checksum(&self) -> bool {
        let mut copy = self.clone();
        copy.checksum = 0;
        copy.update_checksum();
        copy.checksum == self.checksum
    }

    /// Mark filesystem as mounted
    pub fn mark_mounted(&mut self) {
        self.mounted_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.mount_count += 1;
        self.state = FsState::Dirty;
        self.update_checksum();
    }

    /// Mark filesystem as unmounted
    pub fn mark_unmounted(&mut self) {
        self.state = FsState::Clean;
        self.update_checksum();
    }

    /// Update free block count
    pub fn set_free_blocks(&mut self, free_blocks: u64) {
        self.free_blocks = free_blocks;
        self.update_checksum();
    }

    /// Read superblock from storage backend
    ///
    /// # Arguments
    /// * `backend` - Storage backend to read from
    ///
    /// # Errors
    /// * StorageError if read fails
    pub fn read_from_storage<B: StorageBackend>(backend: &mut B) -> StorageResult<Self> {
        log::debug!("Reading superblock from storage");
        let block = backend.read_block(BlockId(0))?;
        let sb = Self::from_block(&block)
            .map_err(|e| {
                log::error!("Failed to deserialize superblock: {}", e);
                StorageError::Backend(format!("Failed to deserialize superblock: {}", e))
            })?;

        if !sb.is_valid() {
            log::error!("Invalid superblock magic or version");
            return Err(StorageError::Backend("Invalid superblock magic or version".to_string()));
        }

        if !sb.verify_checksum() {
            log::error!("Superblock checksum verification failed");
            return Err(StorageError::Backend("Superblock checksum verification failed".to_string()));
        }

        log::info!("Successfully read superblock: {} blocks, {} free", sb.total_blocks, sb.free_blocks);
        Ok(sb)
    }

    /// Write superblock to storage backend
    ///
    /// # Arguments
    /// * `backend` - Storage backend to write to
    ///
    /// # Errors
    /// * StorageError if write fails
    pub fn write_to_storage<B: StorageBackend>(&self, backend: &mut B) -> StorageResult<()> {
        log::debug!("Writing superblock to storage");
        let block = self.to_block();
        backend.write_block(BlockId(0), &block)?;
        backend.sync()?;
        log::info!("Successfully wrote superblock");
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superblock_creation() {
        let sb = Superblock::new(1000, InodeId::root());
        assert_eq!(sb.magic, MAGIC);
        assert_eq!(sb.version, VERSION);
        assert_eq!(sb.total_blocks, 1000);
        assert_eq!(sb.free_blocks, 999);
        assert_eq!(sb.root_inode, InodeId::root());
        assert!(sb.is_valid());
        assert!(sb.verify_checksum());
    }

    #[test]
    fn test_superblock_serialization() {
        let mut sb = Superblock::new(1000, InodeId::root());
        let block = sb.to_block();
        let sb2 = Superblock::from_block(&block).unwrap();

        assert_eq!(sb, sb2);
        assert!(sb2.is_valid());
        assert!(sb2.verify_checksum());
    }

    #[test]
    fn test_superblock_mount_unmount() {
        let mut sb = Superblock::new(1000, InodeId::root());
        assert_eq!(sb.state, FsState::Clean);
        assert_eq!(sb.mount_count, 0);

        sb.mark_mounted();
        assert_eq!(sb.state, FsState::Dirty);
        assert_eq!(sb.mount_count, 1);

        sb.mark_unmounted();
        assert_eq!(sb.state, FsState::Clean);
    }

    #[test]
    fn test_checksum_verification() {
        let mut sb = Superblock::new(1000, InodeId::root());
        assert!(sb.verify_checksum());

        // Corrupt checksum
        sb.checksum = 0;
        assert!(!sb.verify_checksum());

        // Fix it
        sb.update_checksum();
        assert!(sb.verify_checksum());
    }
}