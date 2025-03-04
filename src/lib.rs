mod models;
mod windows_storage;
mod file_extraction;

pub use models::*;
pub use windows_storage::get_disks;
pub use file_extraction::{get_files, get_files_by_pattern, get_recently_modified_files, calculate_directory_size, format_file_size};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_disks() {
        let disks = get_disks();
        assert!(disks.is_ok());
        let disks = disks.unwrap();
        assert!(!disks.is_empty());

        for disk in disks {
            println!("{:?}", disk);
        }
    }
}