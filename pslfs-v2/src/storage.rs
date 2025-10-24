//! # Storage Backend Abstraction for PSLFS V2
//!
//! This module provides the `StorageBackend` trait that allows PSLFS to work with
//! different storage implementations (file-based, memory-based, block devices, etc.).
//!
//! ## Design Goals
//!
//! - **Abstraction**: Clean interface for different storage types
//! - **Performance**: Efficient block-level I/O operations
//! - **Safety**: Type-safe operations with comprehensive error handling
//! - **Testability**: Easy to mock and test with memory backend
//! - **Compatibility**: Support for V0.1 format and future extensions
//!
//! ## Architecture
//!
//! The storage layer is designed around the `StorageBackend` trait, which provides
//! block-level read/write operations. This allows PSLFS to work with:
//!
//! - **FileBackend**: Persistent storage in regular files (V0.1 compatible)
//! - **MemoryBackend**: In-memory storage for testing and temporary filesystems
//! - **BlockDeviceBackend**: Direct block device access (future extension)
//!
//! ## Block Format
//!
//! All storage backends operate on 4KB blocks. The block size is fixed at compile time
//! for simplicity and performance. Each block contains:
//!
//! - **Data**: 4096 bytes of filesystem data
//! - **Metadata**: Implicit structure based on filesystem layer requirements
//!
//! ## Error Handling
//!
//! Storage operations use the `StorageError` enum for comprehensive error reporting:
//!
//! - **Io**: I/O related errors (disk full, permission denied, etc.)
//! - **InvalidBlock**: Block ID out of range
//! - **BlockNotFound**: Block doesn't exist (for sparse backends)
//! - **Backend**: Backend-specific errors
//!
//! ## Usage Example
//!
//! ```rust
//! use pslfs::storage::{FileBackend, MemoryBackend, StorageBackend};
//! use pslfs::{Block, BlockId};
//!
//! // File-based storage (persistent)
//! let mut file_backend = FileBackend::create("/tmp/fs.bin", 1000)?;
//! let data = Block::from_slice(b"Hello");
//! file_backend.write_block(BlockId(0), &data)?;
//! let read_data = file_backend.read_block(BlockId(0))?;
//!
//! // Memory-based storage (for testing)
//! let mut mem_backend = MemoryBackend::new(1000);
//! mem_backend.write_block(BlockId(1), &data)?;
//! let read_data = mem_backend.read_block(BlockId(1))?;
//!
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Performance Considerations
//!
//! - **Block Alignment**: All operations are aligned to 4KB boundaries
//! - **Buffering**: FileBackend uses OS-level buffering for efficiency
//! - **Sync**: Explicit sync operations for crash safety
//! - **Caching**: Higher layers handle caching; storage layer focuses on I/O

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