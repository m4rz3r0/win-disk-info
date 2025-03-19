//! This module provides structures for representing file system entries.
//!
//! It contains the `FileEntry` struct that encapsulates information about files
//! and directories, along with supporting error types and utility methods for
//! working with file system entries.

use std::path::{Path, PathBuf};
use std::{fmt, io};
use chrono::{DateTime, Local};
use walkdir::DirEntry;

/// Represents a file system entry with its metadata.
/// 
/// This structure holds information about a file or directory
/// including its path, name, size, modification time, and extension.
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Complete path to the file or directory
    path: PathBuf,
    /// File or directory name without the path
    name: String,
    /// File extension (if any)
    extension: Option<String>,
    
    /// File size in bytes
    size: u64,
    /// Last modification timestamp
    modified: DateTime<Local>,
}

/// Error type for FileEntry creation failures
#[derive(Debug)]
pub enum FileEntryError {
    /// Error occurred when accessing file metadata
    MetadataError(walkdir::Error),
    /// Error occurred when accessing file timestamps
    TimeError(io::Error),
}

impl FileEntry {
    /// Creates a new FileEntry from a walkdir::DirEntry.
    ///
    /// This method extracts all relevant metadata from the directory entry,
    /// including path, name, size, and timestamps.
    ///
    /// # Arguments
    ///
    /// * `entry` - A reference to a walkdir::DirEntry
    ///
    /// # Returns
    ///
    /// * `Result<Self, FileEntryError>` - A new FileEntry or an error if metadata retrieval fails
    ///
    /// # Examples
    ///
    /// ```
    /// use walkdir::WalkDir;
    /// use win_disk_info::FileEntry;
    ///
    /// for entry_result in WalkDir::new(".").into_iter().filter_map(Result::ok) {
    ///     if let Ok(file_entry) = FileEntry::from_dir_entry(&entry_result) {
    ///         println!("Found: {}, size: {}", file_entry.name(), file_entry.size());
    ///     }
    /// }
    /// ```
    pub fn from_dir_entry(entry: &DirEntry) -> Result<Self, FileEntryError> {
        let path = entry.path().to_path_buf();
        let name = entry.file_name().to_string_lossy().to_string();
        
        // Get metadata safely
        let metadata = entry.metadata()
            .map_err(FileEntryError::MetadataError)?;
        
        let size = metadata.len();
        
        // Convert SystemTime to DateTime<Local> safely
        let modified = metadata.modified()
            .map_err(FileEntryError::TimeError)
            .map(DateTime::<Local>::from)?;
        
        let extension = path.extension()
            .map(|ext| ext.to_string_lossy().to_string());
        
        Ok(FileEntry {
            path,
            name,
            size,
            modified,
            extension,
        })
    }

    /// Returns the file name component of this path
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the extension of this file, if any
    pub fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }

    /// Returns a reference to the complete path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Consumes the FileEntry and returns its path
    pub fn into_path(self) -> PathBuf {
        self.path
    }

    /// Returns the file size in bytes
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Returns the last modification time
    pub fn modified(&self) -> DateTime<Local> {
        self.modified
    }
    
    /// Determines if this is a hidden file
    ///
    /// On Windows, checks the hidden file attribute.
    /// On Unix-like systems, checks if the filename starts with a dot.
    ///
    /// # Returns
    ///
    /// * `bool` - true if the file is hidden, false otherwise
    pub fn is_hidden(&self) -> bool {
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if let Ok(metadata) = std::fs::metadata(&self.path) {
                // Check FILE_ATTRIBUTE_HIDDEN (0x2) on Windows
                return (metadata.file_attributes() & 0x2) != 0;
            }
        }
        
        #[cfg(not(windows))]
        {
            // On Unix-like systems, hidden files start with '.'
            return self.name.starts_with('.');
        }
        
        false
    }
}

/// For backward compatibility with code that uses the From trait
///
/// This implementation provides a way to create a FileEntry from a DirEntry
/// while handling errors internally to avoid propagating them.
impl From<DirEntry> for FileEntry {
    fn from(entry: DirEntry) -> Self {
        Self::from_dir_entry(&entry).unwrap_or_else(|err| {
            // Log the error and return a fallback value
            eprintln!("Error creating FileEntry: {:?}", err);
            let path = entry.path().to_path_buf();
            let name = entry.file_name().to_string_lossy().to_string();
            
            FileEntry {
                path,
                name,
                size: 0,
                modified: Local::now(),
                extension: None,
            }
        })
    }
}

impl fmt::Display for FileEntry {
    /// Formats the `FileEntry` struct for display.
    ///
    /// Provides a detailed representation of the file entry including:
    /// - Name and path
    /// - Type (file/directory) and extension
    /// - Size in appropriate units
    /// - Last modification timestamp
    /// - Hidden status
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Determine if it's a file or directory
        let entry_type = if self.path.is_dir() { "Directory" } else { "File" };
        
        // Format file size in appropriate units
        let (size_value, size_unit) = if self.size >= 1_000_000_000 {
            (self.size as f64 / 1_000_000_000.0, "GB")
        } else if self.size >= 1_000_000 {
            (self.size as f64 / 1_000_000.0, "MB")
        } else if self.size >= 1_000 {
            (self.size as f64 / 1_000.0, "KB")
        } else {
            (self.size as f64, "bytes")
        };
        
        // Format modification time
        let mod_time = self.modified.format("%Y-%m-%d %H:%M:%S");
        
        // Format hidden status
        let hidden_status = if self.is_hidden() { " (Hidden)" } else { "" };
        
        // Write formatted output
        write!(
            f,
            "{}{}\n  Type: {}\n  Path: {}\n  Size: {:.2} {}\n  Modified: {}",
            self.name,
            hidden_status,
            entry_type,
            self.path.display(),
            size_value,
            size_unit,
            mod_time
        )
    }
}