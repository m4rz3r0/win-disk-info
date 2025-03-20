//! This module provides structures for representing disk partitions and file systems.
//!
//! It contains the `Partition` struct that represents a partition on a physical disk,
//! along with the `FileSystem` enum that categorizes different file system types.

use std::{fmt, path::PathBuf};

#[cfg(feature = "serialize")]
use serde::Serialize;

/// Represents various types of file systems with their mount points.
///
/// Each variant contains the path(s) where the file system is mounted.
/// Some file systems like BTRFS can have multiple mount points.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
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
#[cfg_attr(feature = "serialize", derive(Serialize))]
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

impl fmt::Display for FileSystem {
    /// Formats the `FileSystem` enum for display.
    ///
    /// Provides a string representation of the file system type and its mount points.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileSystem::BTRFS(paths) => {
                write!(f, "BTRFS")?;
                if !paths.is_empty() {
                    write!(f, " [")?;
                    for (i, path) in paths.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", path.display())?;
                    }
                    write!(f, "]")?;
                }
                Ok(())
            },
            FileSystem::EXT4(path) => write!(f, "EXT4 [{}]", path.display()),
            FileSystem::NTFS(path) => write!(f, "NTFS [{}]", path.display()),
            FileSystem::FAT32(path) => write!(f, "FAT32 [{}]", path.display()),
            FileSystem::EXFAT(path) => write!(f, "exFAT [{}]", path.display()),
            FileSystem::XFS(path) => write!(f, "XFS [{}]", path.display()),
            FileSystem::ZFS(path) => write!(f, "ZFS [{}]", path.display()),
            FileSystem::NotImplemented(name, path) => write!(f, "{} [{}]", name, path.display()),
            FileSystem::Unknown => write!(f, "Unknown"),
        }
    }
}

impl fmt::Display for Partition {
    /// Formats the `Partition` struct for display.
    ///
    /// Provides a detailed representation of the partition including:
    /// - Name and ID
    /// - File system type and mount point(s)
    /// - Total and available space
    /// - Usage percentage
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Calculate used space and usage percentage
        let used_space = self.total_space - self.available_space;
        let usage_percent = if self.total_space > 0 {
            (used_space as f64 / self.total_space as f64) * 100.0
        } else {
            0.0
        };
        
        // Format space values in appropriate units
        let format_bytes = |bytes: u64| -> (f64, &'static str) {
            if bytes >= 1_099_511_627_776 { // 1 TiB
                (bytes as f64 / 1_099_511_627_776.0, "TiB")
            } else if bytes >= 1_073_741_824 { // 1 GiB
                (bytes as f64 / 1_073_741_824.0, "GiB")
            } else if bytes >= 1_048_576 { // 1 MiB
                (bytes as f64 / 1_048_576.0, "MiB")
            } else if bytes >= 1_024 { // 1 KiB
                (bytes as f64 / 1_024.0, "KiB")
            } else {
                (bytes as f64, "bytes")
            }
        };
        
        let (total_val, total_unit) = format_bytes(self.total_space);
        let (used_val, used_unit) = format_bytes(used_space);
        let (avail_val, avail_unit) = format_bytes(self.available_space);
        
        // Write formatted output
        write!(
            f,
            "Partition {}: {}\n  File System: {}\n  Space: {:.2} {} total, {:.2} {} used ({:.1}%), {:.2} {} free",
            self.id,
            self.name,
            self.file_system,
            total_val, total_unit,
            used_val, used_unit,
            usage_percent,
            avail_val, avail_unit
        )
    }
}