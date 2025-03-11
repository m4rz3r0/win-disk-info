//! # Windows Disk Info
//! 
//! A Rust library for retrieving and analyzing information about disks, 
//! partitions, and files on Windows systems.
//!
//! This library provides functionality to:
//! - Query physical disk information using Windows WMI
//! - List partitions and their properties
//! - Extract file information from directories
//! - Identify file types based on content
//! - Find files with incorrect extensions
//! - Calculate directory sizes
//!
//! ## Example
//!
//! ```no_run
//! use win_disk_info::{get_disks, get_files, identify_files};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Get information about all physical disks
//!     let disks = get_disks()?;
//!     
//!     for disk in disks {
//!         println!("Disk: {} ({} GB)", 
//!             disk.model(), 
//!             disk.size() / 1_000_000_000);
//!     }
//!     
//!     // List files in a directory
//!     let files = get_files("C:\\Documents")?;
//!     
//!     // Identify file types
//!     let identified = identify_files(files);
//!     
//!     Ok(())
//! }
//! ```

mod models;
mod windows_storage;
mod file_extraction;
mod file_identification;

pub use models::*;
pub use windows_storage::get_disks;
pub use file_extraction::{get_files, get_files_by_pattern, get_recently_modified_files, calculate_directory_size, format_file_size};
pub use file_identification::{identify_files, validate_file_extension, find_mismatched_extensions};