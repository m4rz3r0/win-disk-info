//! This module provides structures for representing disk partitions and file systems.
//!
//! It contains the `Partition` struct that represents a partition on a physical disk,
//! along with the `FileSystem` enum that categorizes different file system types.

use std::path::PathBuf;

/// Represents various types of file systems with their mount points.
///
/// Each variant contains the path(s) where the file system is mounted.
/// Some file systems like BTRFS can have multiple mount points.
#[derive(Debug, Clone, PartialEq)]
pub enum FileSystem {
    /// BTRFS file system with potentially multiple mount points
    BTRFS(Vec<PathBuf>),
    /// EXT4 Linux file system with its mount point
    EXT4(PathBuf),
    /// NTFS Windows file system with its mount point
    NTFS(PathBuf),
    /// FAT32 file system with its mount point
    FAT32(PathBuf),
    /// exFAT file system with its mount point
    EXFAT(PathBuf),
    /// XFS file system with its mount point
    XFS(PathBuf),
    /// ZFS file system with its mount point
    ZFS(PathBuf),
    /// Recognized but not fully implemented file system with type name and mount point
    NotImplemented(String, PathBuf),
    /// Unknown or unrecognized file system
    Unknown,
}

/// Represents a logical partition on a physical disk.
///
/// Contains information about a disk partition including its identifier,
/// name, file system type, and space usage statistics.
#[derive(Debug, Clone, PartialEq)]
pub struct Partition {
    /// Unique identifier for this partition
    id: usize,
    /// Descriptive name of the partition (e.g., "C:", "System Reserved")
    name: String,
    /// File system type and mount point(s)
    file_system: FileSystem,
    /// Total capacity of the partition in bytes
    total_space: u64,
    /// Available free space in bytes
    available_space: u64,
}

impl Partition {
    /// Creates a new Partition with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the partition
    /// * `name` - Descriptive name of the partition
    /// * `file_system` - Type of file system and its mount point(s)
    /// * `total_space` - Total capacity in bytes
    /// * `available_space` - Available free space in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use win_disk_info::{Partition, FileSystem};
    /// use std::path::PathBuf;
    ///
    /// let partition = Partition::new(
    ///     0,
    ///     String::from("C:"),
    ///     FileSystem::NTFS(PathBuf::from("C:\\")),
    ///     512_000_000_000,  // 512 GB
    ///     128_000_000_000,  // 128 GB free
    /// );
    /// ```
    pub fn new(
        id: usize,
        name: String,
        file_system: FileSystem,
        total_space: u64,
        available_space: u64,
    ) -> Self {
        Partition {
            id,
            name,
            file_system,
            total_space,
            available_space,
        }
    }

    /// Returns the unique identifier of this partition.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Returns the descriptive name of this partition.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the file system information.
    pub fn file_system(&self) -> &FileSystem {
        &self.file_system
    }

    /// Returns the total capacity of this partition in bytes.
    pub fn total_space(&self) -> u64 {
        self.total_space
    }

    /// Returns the available free space in this partition in bytes.
    pub fn available_space(&self) -> u64 {
        self.available_space
    }
}