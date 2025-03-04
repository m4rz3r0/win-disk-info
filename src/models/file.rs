use std::path::{Path, PathBuf};
use std::io;
use chrono::{DateTime, Local};
use walkdir::DirEntry;

/// Represents a file system entry with its metadata.
/// 
/// This structure holds information about a file or directory
/// including its path, name, size, modification time, and extension.
#[derive(Debug, Clone)]
pub struct FileEntry {
    // Path details
    path: PathBuf,
    name: String,
    extension: Option<String>,
    
    // File metadata
    size: u64,
    modified: DateTime<Local>,
}

/// Error type for FileEntry creation failures
#[derive(Debug)]
pub enum FileEntryError {
    MetadataError(walkdir::Error),
    TimeError(io::Error),
}

impl FileEntry {
    /// Creates a new FileEntry from a walkdir::DirEntry.
    ///
    /// Returns an error if metadata for the entry can't be obtained.
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

// For backward compatibility
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