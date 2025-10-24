//! PSLFS V2 - A modern, educational filesystem implementation in Rust
//!
//! This library provides a complete filesystem implementation with:
//! - Memory-safe operations using Rust's ownership system
//! - Pluggable storage backends (file, memory, block device)
//! - Efficient bitmap-based allocation
//! - Comprehensive error handling
//! - FUSE integration for mounting as a real filesystem

pub mod types;
pub mod storage;
pub mod allocator;

pub use types::*;
pub use storage::*;
pub use allocator::*;