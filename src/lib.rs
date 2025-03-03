mod models;
mod windows_storage;

pub use models::*;
pub use windows_storage::get_disks;

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use wmi::Variant;

    use crate::windows_storage::{
        create_file_system, extract_disk_number, get_bool_value, get_disk_kind, get_string_value,
        get_u64_value, SupportedFileSystem, MEDIA_TYPE_HDD, MEDIA_TYPE_SCM, MEDIA_TYPE_SSD,
    };

    use super::*;

    #[test]
    fn test_extract_disk_number() {
        assert_eq!(extract_disk_number("\\\\.\\PHYSICALDRIVE0"), 0);
        assert_eq!(extract_disk_number("\\\\.\\PHYSICALDRIVE1"), 1);
        assert_eq!(extract_disk_number("\\\\.\\PHYSICALDRIVE10"), 10);
        assert_eq!(extract_disk_number("invalid"), 0);
        assert_eq!(extract_disk_number("\\\\.\\PHYSICALDRIVE"), 0);
        assert_eq!(extract_disk_number("\\\\.\\PHYSICALDRIVEabc"), 0);
    }

    #[test]
    fn test_get_string_value() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), Variant::String("value1".to_string()));
        map.insert("key2".to_string(), Variant::UI4(42));

        assert_eq!(get_string_value(&map, "key1"), Some("value1".to_string()));
        assert_eq!(get_string_value(&map, "key2"), None);
        assert_eq!(get_string_value(&map, "key3"), None);
    }

    #[test]
    fn test_get_u64_value() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), Variant::UI8(1234567890));
        map.insert(
            "key2".to_string(),
            Variant::String("not_a_number".to_string()),
        );

        assert_eq!(get_u64_value(&map, "key1"), Some(1234567890));
        assert_eq!(get_u64_value(&map, "key2"), None);
        assert_eq!(get_u64_value(&map, "key3"), None);
    }

    #[test]
    fn test_get_bool_value() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), Variant::Bool(true));
        map.insert("key2".to_string(), Variant::Bool(false));
        map.insert(
            "key3".to_string(),
            Variant::String("not_a_bool".to_string()),
        );

        assert_eq!(get_bool_value(&map, "key1"), Some(true));
        assert_eq!(get_bool_value(&map, "key2"), Some(false));
        assert_eq!(get_bool_value(&map, "key3"), None);
        assert_eq!(get_bool_value(&map, "key4"), None);
    }

    #[test]
    fn test_supported_filesystem_from() {
        assert_eq!(SupportedFileSystem::from("NTFS"), SupportedFileSystem::NTFS);
        assert_eq!(
            SupportedFileSystem::from("FAT32"),
            SupportedFileSystem::FAT32
        );
        assert_eq!(
            SupportedFileSystem::from("exFAT"),
            SupportedFileSystem::EXFAT
        );

        match SupportedFileSystem::from("XFS") {
            SupportedFileSystem::NotImplemented(fs) => assert_eq!(fs, "XFS"),
            _ => panic!("Expected NotImplemented variant"),
        }
    }

    #[test]
    fn test_create_file_system() {
        match create_file_system("NTFS", "C:\\") {
            Some(FileSystem::NTFS(path)) => assert_eq!(path, PathBuf::from("C:\\")),
            _ => panic!("Expected NTFS file system"),
        }

        match create_file_system("FAT32", "D:\\") {
            Some(FileSystem::FAT32(path)) => assert_eq!(path, PathBuf::from("D:\\")),
            _ => panic!("Expected FAT32 file system"),
        }

        match create_file_system("exFAT", "E:\\") {
            Some(FileSystem::EXFAT(path)) => assert_eq!(path, PathBuf::from("E:\\")),
            _ => panic!("Expected EXFAT file system"),
        }

        match create_file_system("EXT4", "F:\\") {
            Some(FileSystem::NotImplemented(fs, path)) => {
                assert_eq!(fs, "EXT4");
                assert_eq!(path, PathBuf::from("F:\\"));
            }
            _ => panic!("Expected NotImplemented file system"),
        }
    }

    #[test]
    fn test_get_disk_kind() {
        let mut map = HashMap::new();

        // Test HDD
        map.insert("Kind".to_string(), Variant::UI2(MEDIA_TYPE_HDD));
        assert_eq!(get_disk_kind(&map), Some(DiskKind::HDD));

        // Test SSD
        map.insert("Kind".to_string(), Variant::UI2(MEDIA_TYPE_SSD));
        assert_eq!(get_disk_kind(&map), Some(DiskKind::SSD));

        // Test SCM
        map.insert("Kind".to_string(), Variant::UI2(MEDIA_TYPE_SCM));
        assert_eq!(get_disk_kind(&map), Some(DiskKind::SCM));

        // Test unknown type
        map.insert("Kind".to_string(), Variant::UI2(10));
        if let Some(DiskKind::Unknown(kind)) = get_disk_kind(&map) {
            assert_eq!(kind, 10);
        } else {
            panic!("Expected Unknown disk kind");
        }

        // Test missing kind
        map.remove("Kind");
        if let Some(DiskKind::Unknown(kind)) = get_disk_kind(&map) {
            assert_eq!(kind, -1);
        } else {
            panic!("Expected Unknown disk kind with value -1");
        }
    }
}
