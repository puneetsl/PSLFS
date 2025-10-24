//! # Unified Error Types for PSLFS V2
//!
//! This module provides a comprehensive error handling system for the filesystem.
//! All operations return `Result<T, FsError>` for consistent error propagation.
//!
//! ## Error Hierarchy
//!
//! - **FsError**: Top-level error enum covering all filesystem operations
//! - **StorageError**: Low-level storage backend errors
//! - **AllocatorError**: Block allocation errors
//! - **InodeError**: Inode management errors (future)
//! - **PathError**: Path resolution errors (future)
//!
//! ## Usage
//!
//! ```rust
//! use pslfs::{Filesystem, FsError};
//!
//! match Filesystem::open("/tmp/fs") {
//!     Ok(fs) => println!("Opened filesystem"),
//!     Err(FsError::Storage(e)) => println!("Storage error: {}", e),
//!     Err(FsError::NotFound) => println!("Filesystem not found"),
//!     Err(e) => println!("Other error: {}", e),
//! }
//! ```
//!
//! ## Error Conversion
//!
//! All lower-level errors automatically convert to `FsError` using `From` traits.
//! This ensures consistent error handling throughout the codebase.

use crate::storage::StorageError;
use crate::allocator::AllocatorError;

/// Top-level filesystem error type
#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Allocator error: {0}")]
    Allocator(#[from] AllocatorError),

    #[error("Filesystem not found")]
    NotFound,

    #[error("Filesystem already exists")]
    AlreadyExists,

    #[error("Invalid filesystem format")]
    InvalidFormat,

    #[error("Filesystem is corrupted")]
    Corrupted,

    #[error("Operation not supported")]
    NotSupported,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Disk full")]
    DiskFull,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, FsError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageError;
    use crate::allocator::AllocatorError;

    #[test]
    fn test_error_conversion() {
        // Test StorageError conversion
        let storage_err = StorageError::InvalidBlock(100);
        let fs_err: FsError = storage_err.into();
        assert!(matches!(fs_err, FsError::Storage(_)));

        // Test AllocatorError conversion
        let alloc_err = AllocatorError::NoSpace;
        let fs_err: FsError = alloc_err.into();
        assert!(matches!(fs_err, FsError::Allocator(_)));

        // Test Io error conversion
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let fs_err: FsError = io_err.into();
        assert!(matches!(fs_err, FsError::Io(_)));
    }

    #[test]
    fn test_error_display() {
        let err = FsError::NotFound;
        assert_eq!(format!("{}", err), "Filesystem not found");

        let err = FsError::InvalidArgument("bad path".to_string());
        assert_eq!(format!("{}", err), "Invalid argument: bad path");
    }
}