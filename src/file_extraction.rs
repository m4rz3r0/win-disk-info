use std::fs;
use std::time::SystemTime;
use sha256::try_digest;
use walkdir::WalkDir;

use crate::FileEntry;

pub fn get_files(path: &str) -> Result<Vec<FileEntry>, walkdir::Error> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.push(FileEntry::from(entry));
        }
    }
    Ok(files)
}

/// Gets files that match a specific pattern
pub fn get_files_by_pattern(path: &str, pattern: &str) -> Result<Vec<FileEntry>, walkdir::Error> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_string_lossy();
            if file_name.contains(pattern) {
                files.push(FileEntry::from(entry));
            }
        }
    }
    Ok(files)
}

/// Gets the largest files in a directory
pub fn get_largest_files(path: &str, limit: usize) -> Result<Vec<FileEntry>, walkdir::Error> {
    let mut files: Vec<FileEntry> = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.push(FileEntry::from(entry));
        }
    }

    // Sort by size in descending order
    files.sort_by(|a, b| b.size().cmp(&a.size()));

    // Limit the number of results
    Ok(files.into_iter().take(limit).collect())
}

/// Gets recently modified files
pub fn get_recently_modified_files(
    path: &str,
    days: u64,
) -> Result<Vec<FileEntry>, Box<dyn std::error::Error>> {
    let now = SystemTime::now();
    let duration = std::time::Duration::from_secs(days * 24 * 60 * 60);
    let cutoff = now.checked_sub(duration).unwrap_or(now);

    let mut files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let metadata = fs::metadata(entry.path())?;
            if let Ok(modified) = metadata.modified() {
                if modified >= cutoff {
                    files.push(FileEntry::from(entry));
                }
            }
        }
    }

    Ok(files)
}

/// Calculates the total size of a directory
pub fn calculate_directory_size(path: &str) -> Result<u64, walkdir::Error> {
    let mut total_size: u64 = 0;
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
    }
    Ok(total_size)
}

/// Finds duplicate files based on their content
pub fn find_duplicate_files(path: &str) -> Result<Vec<Vec<FileEntry>>, Box<dyn std::error::Error>> {
    use std::collections::HashMap;

    // First step: group files by size to reduce comparisons
    let mut size_map: HashMap<u64, Vec<FileEntry>> = HashMap::new();

    // Collect files by size
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let file_entry = FileEntry::from(entry);
            let size = file_entry.size();

            // Ignore empty files
            if size > 0 {
                size_map.entry(size).or_insert_with(Vec::new).push(file_entry);
            }
        }
    }

    // Second step: calculate hash only for files with the same size
    let mut hash_map: HashMap<String, Vec<FileEntry>> = HashMap::new();
    
    for (_size, files) in size_map.into_iter().filter(|(_, files)| files.len() > 1) {
        for file in files {
            // Calculate content hash
            let hash = try_digest(file.path()).unwrap();
            
            hash_map.entry(hash).or_insert_with(Vec::new).push(file);
        }
    }

    // Filter only sets with duplicates
    let duplicates: Vec<Vec<FileEntry>> = hash_map
        .into_iter()
        .filter(|(_, entries)| entries.len() > 1)
        .map(|(_, entries)| entries)
        .collect();

    Ok(duplicates)
}

/// Formats file size into human-readable units
pub fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} bytes", size)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;
    
    use super::*;
    
    // Helper function to create a test directory with files
    fn setup_test_directory() -> tempfile::TempDir {
        let dir = tempdir().expect("Could not create temporary directory");
        
        // Create some test files
        let file_paths = [
            (dir.path().join("small.txt"), 100),
            (dir.path().join("medium.txt"), 2000),
            (dir.path().join("large.txt"), 5000),
            (dir.path().join("test_file.dat"), 1500),
            (dir.path().join("another_test.dat"), 1500),
        ];
        
        for (path, size) in file_paths.iter() {
            let mut file = File::create(path).expect("Could not create file");
            let data = vec![b'A'; *size];
            file.write_all(&data).expect("Could not write to file");
        }
        
        // Create a subdirectory with files
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).expect("Could not create subdirectory");
        
        let subdir_file = subdir.join("subfile.txt");
        let mut file = File::create(&subdir_file).expect("Could not create file in subdirectory");
        let data = vec![b'B'; 1000];
        file.write_all(&data).expect("Could not write to subdirectory file");
        
        dir
    }
    
    #[test]
    fn test_get_files() {
        let temp_dir = setup_test_directory();
        let result = get_files(temp_dir.path().to_str().unwrap());
        
        assert!(result.is_ok());
        let files = result.unwrap();
        
        // Should have 6 files (5 in the root directory and 1 in the subdirectory)
        assert_eq!(files.len(), 6);
    }
    
    #[test]
    fn test_get_files_by_pattern() {
        let temp_dir = setup_test_directory();
        let result = get_files_by_pattern(temp_dir.path().to_str().unwrap(), "test");
        
        assert!(result.is_ok());
        let files = result.unwrap();
        
        // Should have 2 files with "test" in the name
        assert_eq!(files.len(), 2);
        
        // Verify that the names contain "test"
        for file in files {
            assert!(file.name().to_string_lossy().contains("test"));
        }
    }
    
    #[test]
    fn test_get_largest_files() {
        let temp_dir = setup_test_directory();
        let result = get_largest_files(temp_dir.path().to_str().unwrap(), 3);
        
        assert!(result.is_ok());
        let files = result.unwrap();
        
        // Should be 3 files (limit = 3)
        assert_eq!(files.len(), 3);
        
        // The first file should be the largest (large.txt)
        assert_eq!(files[0].size(), 5000);
        assert!(files[0].name().to_string_lossy().contains("large"));
        
        // Verify they are sorted by size
        assert!(files[0].size() >= files[1].size());
        assert!(files[1].size() >= files[2].size());
    }
    
    #[test]
    fn test_recently_modified_files() {
        let temp_dir = setup_test_directory();
        let result = get_recently_modified_files(temp_dir.path().to_str().unwrap(), 1);
        
        assert!(result.is_ok());
        let files = result.unwrap();
        
        // All files should be recent (less than a day old)
        assert_eq!(files.len(), 6);
    }
    
    #[test]
    fn test_calculate_directory_size() {
        let temp_dir = setup_test_directory();
        let result = calculate_directory_size(temp_dir.path().to_str().unwrap());
        
        assert!(result.is_ok());
        let size = result.unwrap();
        
        // The total size should be the sum of all files (100 + 2000 + 5000 + 1500 + 1500 + 1000)
        assert_eq!(size, 11100);
    }
    
    #[test]
    fn test_find_duplicate_files() {
        let temp_dir = setup_test_directory();
        let result = find_duplicate_files(temp_dir.path().to_str().unwrap());
        
        assert!(result.is_ok());
        let duplicates = result.unwrap();
        
        // Should have 1 set of duplicates (test_file.dat and another_test.dat have the same size)
        assert_eq!(duplicates.len(), 1);
        
        // Each set should have at least 2 files
        for group in &duplicates {
            assert!(group.len() >= 2);
        }
    }
    
    #[test]
    fn test_format_file_size() {
        // Test bytes
        assert_eq!(format_file_size(100), "100 bytes");
        
        // Test KB
        assert_eq!(format_file_size(2048), "2.00 KB");
        
        // Test MB
        let mb_size = 1024 * 1024 * 5;  // 5MB
        assert_eq!(format_file_size(mb_size), "5.00 MB");
        
        // Test GB
        let gb_size = 1024 * 1024 * 1024 * 2;  // 2GB
        assert_eq!(format_file_size(gb_size), "2.00 GB");
    }
}