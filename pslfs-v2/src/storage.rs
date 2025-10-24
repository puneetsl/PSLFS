//! Storage backend abstraction for PSLFS V2
//!
//! This module provides the `StorageBackend` trait that allows PSLFS to work with
//! different storage implementations (file-based, memory-based, block devices, etc.).

use crate::{Block, BlockId};

/// Errors that can occur during storage operations
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid block ID: {0}")]
    InvalidBlock(u64),

    #[error("Block not found: {0}")]
    BlockNotFound(BlockId),

    #[error("Storage backend error: {0}")]
    Backend(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Trait for storage backends that provide block-level read/write operations
pub trait StorageBackend: Send + Sync {
    /// Read a block from storage
    ///
    /// # Arguments
    /// * `block_id` - The ID of the block to read
    ///
    /// # Returns
    /// The block data
    ///
    /// # Errors
    /// * `InvalidBlock` if block_id is out of range
    /// * `Io` for I/O related errors
    fn read_block(&mut self, block_id: BlockId) -> Result<Block>;

    /// Write a block to storage
    ///
    /// # Arguments
    /// * `block_id` - The ID of the block to write
    /// * `data` - The block data to write
    ///
    /// # Errors
    /// * `InvalidBlock` if block_id is out of range
    /// * `Io` for I/O related errors
    fn write_block(&mut self, block_id: BlockId, data: &Block) -> Result<()>;

    /// Sync all pending writes to persistent storage
    ///
    /// # Errors
    /// * `Io` for I/O related errors
    fn sync(&mut self) -> Result<()>;

    /// Get the total number of blocks available
    fn total_blocks(&self) -> u64;

    /// Get the size of each block in bytes
    fn block_size(&self) -> usize {
        Block::SIZE
    }
}

/// File-based storage backend that stores blocks in regular files
pub mod file;
/// In-memory storage backend for testing and temporary filesystems
pub mod memory;

pub use file::FileBackend;
pub use memory::MemoryBackend;