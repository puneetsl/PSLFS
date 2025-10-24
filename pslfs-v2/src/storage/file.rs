//! File-based storage backend
//!
//! This backend stores filesystem blocks in regular files on disk.
//! It provides persistent storage compatible with V0.1 format.

use super::{StorageBackend, StorageError, Result, Block, BlockId};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;

/// File-based storage backend
pub struct FileBackend {
    file: File,
    total_blocks: u64,
    block_size: usize,
}

impl FileBackend {
    /// Create a new filesystem file with the given size
    ///
    /// # Arguments
    /// * `path` - Path where the filesystem file will be created
    /// * `total_blocks` - Number of blocks to allocate
    ///
    /// # Errors
    /// * `Io` if file creation or writing fails
    pub fn create<P: AsRef<Path>>(path: P, total_blocks: u64) -> Result<Self> {
        let block_size = Block::SIZE;
        let _file_size = total_blocks * block_size as u64;

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        // Pre-allocate space by writing zeros
        let zero_block = vec![0u8; block_size];
        for _ in 0..total_blocks {
            file.write_all(&zero_block)?;
        }
        file.sync_all()?;

        Ok(Self {
            file,
            total_blocks,
            block_size,
        })
    }

    /// Open an existing filesystem file
    ///
    /// # Arguments
    /// * `path` - Path to the existing filesystem file
    ///
    /// # Errors
    /// * `Io` if file opening fails
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;

        let metadata = file.metadata()?;
        let file_size = metadata.len();
        let block_size = Block::SIZE as u64;
        let total_blocks = file_size / block_size;

        if file_size % block_size != 0 {
            return Err(StorageError::Backend(
                format!("File size {} is not a multiple of block size {}", file_size, block_size)
            ));
        }

        Ok(Self {
            file,
            total_blocks,
            block_size: Block::SIZE,
        })
    }
}

impl StorageBackend for FileBackend {
    fn read_block(&mut self, block_id: BlockId) -> Result<Block> {
        if block_id.value() >= self.total_blocks {
            return Err(StorageError::InvalidBlock(block_id.value()));
        }

        let offset = block_id.value() * self.block_size as u64;
        let mut block = Block::new();

        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(block.as_mut_slice())?;

        Ok(block)
    }

    fn write_block(&mut self, block_id: BlockId, data: &Block) -> Result<()> {
        if block_id.value() >= self.total_blocks {
            return Err(StorageError::InvalidBlock(block_id.value()));
        }

        let offset = block_id.value() * self.block_size as u64;

        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(data.as_slice())?;

        Ok(())
    }

    fn sync(&mut self) -> Result<()> {
        self.file.sync_all()?;
        Ok(())
    }

    fn total_blocks(&self) -> u64 {
        self.total_blocks
    }

    fn block_size(&self) -> usize {
        self.block_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_file_backend_create() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.psl");

        let backend = FileBackend::create(&path, 100).unwrap();
        assert_eq!(backend.total_blocks(), 100);
        assert_eq!(backend.block_size(), Block::SIZE);
    }

    #[test]
    fn test_file_backend_read_write() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.psl");
        let mut backend = FileBackend::create(&path, 10).unwrap();

        // Write block
        let data = Block::from_slice(b"Hello, World!");
        backend.write_block(BlockId(5), &data).unwrap();

        // Read block
        let read_data = backend.read_block(BlockId(5)).unwrap();
        assert_eq!(&read_data.data[..13], b"Hello, World!");
    }

    #[test]
    fn test_file_backend_bounds() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.psl");
        let mut backend = FileBackend::create(&path, 10).unwrap();

        // Should fail: out of bounds
        let result = backend.read_block(BlockId(100));
        assert!(matches!(result, Err(StorageError::InvalidBlock(_))));
    }

    #[test]
    fn test_file_backend_open() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.psl");

        // Create file
        {
            let _backend = FileBackend::create(&path, 50).unwrap();
        }

        // Open existing file
        let backend = FileBackend::open(&path).unwrap();
        assert_eq!(backend.total_blocks(), 50);
    }
}