//! In-memory storage backend
//!
//! This backend stores blocks in memory and is primarily used for testing
//! and temporary filesystems that don't need persistence.

use super::{StorageBackend, StorageError, Result, Block, BlockId};
use std::collections::HashMap;

/// In-memory storage backend
pub struct MemoryBackend {
    blocks: HashMap<u64, Block>,
    total_blocks: u64,
}

impl MemoryBackend {
    /// Create a new in-memory storage backend
    ///
    /// # Arguments
    /// * `total_blocks` - Maximum number of blocks that can be allocated
    pub fn new(total_blocks: u64) -> Self {
        Self {
            blocks: HashMap::new(),
            total_blocks,
        }
    }

    /// Create a new in-memory backend with pre-allocated blocks
    ///
    /// # Arguments
    /// * `total_blocks` - Number of blocks to pre-allocate with zeros
    pub fn with_capacity(total_blocks: u64) -> Self {
        let mut blocks = HashMap::new();
        for i in 0..total_blocks {
            blocks.insert(i, Block::new());
        }

        Self {
            blocks,
            total_blocks,
        }
    }
}

impl StorageBackend for MemoryBackend {
    fn read_block(&mut self, block_id: BlockId) -> Result<Block> {
        if block_id.value() >= self.total_blocks {
            return Err(StorageError::InvalidBlock(block_id.value()));
        }

        match self.blocks.get(&block_id.value()) {
            Some(block) => Ok(block.clone()),
            None => Ok(Block::new()), // Return zero block if not allocated
        }
    }

    fn write_block(&mut self, block_id: BlockId, data: &Block) -> Result<()> {
        if block_id.value() >= self.total_blocks {
            return Err(StorageError::InvalidBlock(block_id.value()));
        }

        self.blocks.insert(block_id.value(), data.clone());
        Ok(())
    }

    fn sync(&mut self) -> Result<()> {
        // No-op for memory backend
        Ok(())
    }

    fn total_blocks(&self) -> u64 {
        self.total_blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_backend_new() {
        let mut backend = MemoryBackend::new(100);
        assert_eq!(backend.total_blocks(), 100);

        // Should return zero block for unallocated blocks
        let block = backend.read_block(BlockId(50)).unwrap();
        assert!(block.data.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_memory_backend_with_capacity() {
        let mut backend = MemoryBackend::with_capacity(10);
        assert_eq!(backend.total_blocks(), 10);

        // All blocks should be pre-allocated
        for i in 0..10 {
            let block = backend.read_block(BlockId(i)).unwrap();
            assert!(block.data.iter().all(|&b| b == 0));
        }
    }

    #[test]
    fn test_memory_backend_read_write() {
        let mut backend = MemoryBackend::new(10);

        // Write block
        let data = Block::from_slice(b"Test data");
        backend.write_block(BlockId(3), &data).unwrap();

        // Read back
        let read = backend.read_block(BlockId(3)).unwrap();
        assert_eq!(&read.data[..9], b"Test data");
    }

    #[test]
    fn test_memory_backend_bounds() {
        let mut backend = MemoryBackend::new(10);

        // Should fail: out of bounds
        let result = backend.read_block(BlockId(100));
        assert!(matches!(result, Err(StorageError::InvalidBlock(_))));

        let data = Block::new();
        let result = backend.write_block(BlockId(100), &data);
        assert!(matches!(result, Err(StorageError::InvalidBlock(_))));
    }

    #[test]
    fn test_memory_backend_sync() {
        let mut backend = MemoryBackend::new(10);

        // Sync should always succeed for memory backend
        assert!(backend.sync().is_ok());
    }
}