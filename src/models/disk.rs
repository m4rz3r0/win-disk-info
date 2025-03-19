//! This module provides structures for representing physical disk information.
//!
//! It contains the primary `Disk` struct and supporting enum types that model
//! the characteristics and state of storage devices in the system.

use core::fmt;

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

impl fmt::Display for Disk {
    /// Formats the `Disk` struct for display.
    ///
    /// This implementation provides a comprehensive multi-line view of the disk,
    /// including device name, model, capacity, type, serial number, and detailed
    /// information about each partition on the disk.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format disk size in appropriate units
        let (size_value, size_unit) = if self.size >= 1_099_511_627_776 {
            (self.size as f64 / 1_099_511_627_776.0, "TiB")
        } else if self.size >= 1_073_741_824 {
            (self.size as f64 / 1_073_741_824.0, "GiB")
        } else if self.size >= 1_048_576 {
            (self.size as f64 / 1_048_576.0, "MiB")
        } else if self.size >= 1_024 {
            (self.size as f64 / 1_024.0, "KiB")
        } else {
            (self.size as f64, "bytes")
        };

        // Format disk kind as a string
        let kind_str = match &self.kind {
            DiskKind::HDD => "HDD",
            DiskKind::SSD => "SSD",
            DiskKind::SCM => "SCM",
            DiskKind::Unknown(val) => return write!(f, "Unknown Disk Type ({})", val),
        };

        // Write basic disk information
        write!(
            f,
            "{}\n  Device: {}\n  Type: {}{}\n  Capacity: {:.2} {}\n  Serial: {}\n  Partitions: {}",
            self.model,
            self.device_name,
            kind_str,
            if self.removable { " (Removable)" } else { "" },
            size_value,
            size_unit,
            if self.serial.is_empty() { "N/A" } else { &self.serial },
            self.partitions.len()
        )?;

        // Calculate total allocated space
        let total_allocated: u64 = self.partitions
            .iter()
            .map(|p| p.total_space())
            .sum();

        // If there are partitions, include their details
        if !self.partitions.is_empty() {
            writeln!(f, "\n\nPartition Details:")?;
            
            for (i, partition) in self.partitions.iter().enumerate() {
                // Add separator between partitions except before the first one
                if i > 0 {
                    writeln!(f, "\n  {}", "-".repeat(50))?;
                }
                
                // Indent and format each partition
                let partition_str = format!("{}", partition);
                let indented_lines: Vec<String> = partition_str
                    .lines()
                    .enumerate()
                    .map(|(j, line)| {
                        if j == 0 {
                            format!("  {}", line)
                        } else {
                            format!("    {}", line)
                        }
                    })
                    .collect();
                
                writeln!(f, "{}", indented_lines.join("\n"))?;
            }
            
            // Calculate and display unallocated space if any
            let unallocated = self.size as u64 - total_allocated;
            if unallocated > 1024 { // Only show if significant
                let (unalloc_val, unalloc_unit) = if unallocated >= 1_099_511_627_776 {
                    (unallocated as f64 / 1_099_511_627_776.0, "TiB")
                } else if unallocated >= 1_073_741_824 {
                    (unallocated as f64 / 1_073_741_824.0, "GiB")
                } else if unallocated >= 1_048_576 {
                    (unallocated as f64 / 1_048_576.0, "MiB")
                } else if unallocated >= 1_024 {
                    (unallocated as f64 / 1_024.0, "KiB")
                } else {
                    (unallocated as f64, "bytes")
                };
                
                writeln!(f, "\n  Unallocated space: {:.2} {}", unalloc_val, unalloc_unit)?;
            }
        }
        
        Ok(())
    }
}