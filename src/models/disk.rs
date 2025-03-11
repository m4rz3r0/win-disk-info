//! This module provides structures for representing physical disk information.
//!
//! It contains the primary `Disk` struct and supporting enum types that model
//! the characteristics and state of storage devices in the system.

use crate::Partition;

/// Represents the physical type of a storage device.
///
/// This enum categorizes disks by their underlying storage technology.
#[derive(Debug, Clone, PartialEq)]
pub enum DiskKind {
    /// Hard Disk Drive - traditional mechanical storage
    HDD,
    /// Solid State Drive - flash-based storage
    SSD,
    /// Storage Class Memory - advanced persistent memory technology
    SCM,
    /// Unknown disk type with a media type identifier value
    Unknown(isize),
}

impl Default for DiskKind {
    /// Provides a default value for `DiskKind` (Unknown with value -1).
    ///
    /// This is used when the disk type cannot be determined.
    fn default() -> Self {
        DiskKind::Unknown(-1)
    }
}

/// Represents a physical storage device in the system.
///
/// The `Disk` struct contains comprehensive information about a storage device,
/// including its hardware details and associated partitions.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Disk {
    /// Physical device identifier (e.g., "\\\\.\\PHYSICALDRIVE0")
    device_name: String,
    /// Manufacturer and product name
    model: String,
    /// Unique hardware serial number
    serial: String,
    /// The physical type of storage (HDD, SSD, etc.)
    kind: DiskKind,
    /// Total capacity in bytes
    size: usize,
    /// Whether the disk is removable media
    removable: bool,
    /// List of partitions on this disk
    partitions: Vec<Partition>,
}

impl Disk {
    /// Creates a new instance of `Disk` with all specified parameters.
    ///
    /// # Arguments
    ///
    /// * `device_name` - Physical device identifier
    /// * `model` - Manufacturer and product model
    /// * `serial` - Unique hardware serial number
    /// * `kind` - Type of disk (HDD/SSD/SCM/Unknown)
    /// * `size` - Total capacity in bytes
    /// * `removable` - Whether the disk is removable
    /// * `partitions` - List of partitions on this disk
    ///
    /// # Examples
    ///
    /// ```
    /// use win_disk_info::{Disk, DiskKind};
    ///
    /// let disk = Disk::new(
    ///     String::from("\\\\.\\PHYSICALDRIVE0"),
    ///     String::from("Samsung SSD 970 EVO Plus 1TB"),
    ///     String::from("S4EWNX0M123456"),
    ///     DiskKind::SSD,
    ///     1000204886016,  // 1TB in bytes
    ///     false,
    ///     vec![],
    /// );
    /// ```
    pub fn new(
        device_name: String,
        model: String,
        serial: String,
        kind: DiskKind,
        size: usize,
        removable: bool,
        partitions: Vec<Partition>,
    ) -> Disk {
        Disk {
            device_name,
            model,
            serial,
            kind,
            size,
            removable,
            partitions,
        }
    }

    /// Returns the physical device identifier.
    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    /// Returns the manufacturer and product model.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Returns the unique hardware serial number.
    pub fn serial(&self) -> &str {
        &self.serial
    }

    /// Returns the disk type (HDD/SSD/SCM/Unknown).
    pub fn kind(&self) -> &DiskKind {
        &self.kind
    }

    /// Returns whether the disk is removable media.
    pub fn removable(&self) -> bool {
        self.removable
    }

    /// Returns the total disk capacity in bytes.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns a slice containing all partitions on this disk.
    pub fn partitions(&self) -> &[Partition] {
        &self.partitions
    }
}