use crate::{Disk, DiskError, DiskKind, FileSystem, Partition};
use std::collections::HashMap;
use wmi::{COMLibrary, Variant, WMIConnection};

/// Constants for WMI queries and paths
const WMI_STORAGE_NAMESPACE: &str = "ROOT\\Microsoft\\Windows\\Storage";
const REMOVABLE_MEDIA_CAPABILITY: &str = "Supports Removable Media";

/// Media type constants
pub(super) const MEDIA_TYPE_HDD: u16 = 3;
pub(super) const MEDIA_TYPE_SSD: u16 = 4;
pub(super) const MEDIA_TYPE_SCM: u16 = 5;

/// Supported file systems
#[derive(Debug, PartialEq)]
pub(super) enum SupportedFileSystem {
    NTFS,
    FAT32,
    EXFAT,
    NotImplemented(String),
}

impl From<&str> for SupportedFileSystem {
    fn from(s: &str) -> Self {
        match s {
            "NTFS" => Self::NTFS,
            "FAT32" => Self::FAT32,
            "exFAT" => Self::EXFAT,
            _ => Self::NotImplemented(s.to_string()),
        }
    }
}

/// Update disk information from storage namespace
fn update_disk_info(
    wmi_storage_con: &WMIConnection,
    disk_info: &mut HashMap<String, Variant>,
    device_id: &str,
) -> Result<(), DiskError> {
    let query = format!(
        "SELECT * FROM MSFT_PhysicalDisk WHERE DeviceId = '{}'",
        extract_disk_number(device_id)
    );

    let results: Vec<HashMap<String, Variant>> = wmi_storage_con.raw_query(query)?;

    if let Some(storage_info) = results.first() {
        // Update model if available
        if let Some(Variant::String(model)) = storage_info.get("Model") {
            disk_info.insert("Model".to_string(), Variant::String(model.clone()));
        }

        // Update serial number from FruId if available
        if let Some(Variant::String(fru_id)) = storage_info.get("FruId") {
            disk_info.insert("SerialNumber".to_string(), Variant::String(fru_id.clone()));
        }

        // Update media type/kind if available
        if let Some(Variant::UI2(media_type)) = storage_info.get("MediaType") {
            disk_info.insert("Kind".to_string(), Variant::UI2(*media_type));
        }
    }

    // Check for removable media capability
    if let Some(Variant::Array(capabilities)) = disk_info.get("CapabilityDescriptions") {
        let is_removable =
            capabilities.contains(&Variant::String(REMOVABLE_MEDIA_CAPABILITY.to_string()));
        disk_info.insert("Removable".to_string(), Variant::Bool(is_removable));
    }

    Ok(())
}

/// Get disk kind from media type
pub fn get_disk_kind(disk_info: &HashMap<String, Variant>) -> Option<DiskKind> {
    match disk_info.get("Kind") {
        Some(Variant::UI2(media_type)) => match *media_type {
            MEDIA_TYPE_HDD => Some(DiskKind::HDD),
            MEDIA_TYPE_SSD => Some(DiskKind::SSD),
            MEDIA_TYPE_SCM => Some(DiskKind::SCM),
            kind => Some(DiskKind::Unknown(kind as isize)),
        },
        _ => Some(DiskKind::Unknown(-1_isize)),
    }
}

/// Get logical disk information for a partition
fn get_logical_disk(
    wmi_con: &WMIConnection,
    partition_id: &str,
) -> Result<Option<HashMap<String, Variant>>, DiskError> {
    let query = format!(
        "ASSOCIATORS OF {{Win32_DiskPartition.DeviceID='{}'}} \
         WHERE AssocClass=Win32_LogicalDiskToPartition",
        partition_id
    );

    let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(query)?;
    Ok(results.into_iter().next())
}

// Additional helper functions

/// Extract disk number from device ID
pub(super) fn extract_disk_number(device_id: &str) -> u32 {
    device_id
        .split('\\')
        .last()
        .and_then(|s| s.trim_start_matches("PHYSICALDRIVE").parse().ok())
        .unwrap_or(0)
}

/// Get all disk information from the system
pub fn get_disks() -> Result<Vec<Disk>, DiskError> {
    let com_con = COMLibrary::new()?;
    let wmi_storage_con = WMIConnection::with_namespace_path(WMI_STORAGE_NAMESPACE, com_con)?;
    let wmi_con = WMIConnection::new(com_con)?;

    let disks_wmi: Vec<HashMap<String, Variant>> =
        wmi_con.raw_query("SELECT * FROM Win32_DiskDrive")?;

    let mut partition_count = 0;
    let disks = disks_wmi
        .iter()
        .filter_map(|disk_wmi| {
            process_disk(&wmi_con, &wmi_storage_con, disk_wmi, &mut partition_count)
        })
        .collect();

    Ok(disks)
}

/// Process a single disk from WMI data
fn process_disk(
    wmi_con: &WMIConnection,
    wmi_storage_con: &WMIConnection,
    disk_wmi: &HashMap<String, Variant>,
    partition_count: &mut usize,
) -> Option<Disk> {
    let mut disk_info = disk_wmi.clone();

    // Update disk information from storage namespace
    if let Some(device_id) = get_string_value(&disk_info, "DeviceID") {
        update_disk_info(wmi_storage_con, &mut disk_info, &device_id).ok()?;
    }

    // Get disk properties
    let device_name = get_string_value(&disk_info, "Caption")?
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let model = get_string_value(&disk_info, "Model")?;
    let serial = get_string_value(&disk_info, "SerialNumber")?;
    let kind = get_disk_kind(&disk_info)?;
    let size = get_u64_value(&disk_info, "Size")? as usize;
    let removable = get_bool_value(&disk_info, "Removable")?;

    // Get partitions
    let device_id = get_string_value(&disk_info, "DeviceID")?;
    let partitions = get_partitions(wmi_con, &device_id, partition_count).ok()?;

    Some(Disk::new(
        device_name,
        model,
        serial,
        kind,
        size,
        removable,
        partitions,
    ))
}

/// Get partitions for a disk
fn get_partitions(
    wmi_con: &WMIConnection,
    device_id: &str,
    partition_count: &mut usize,
) -> Result<Vec<Partition>, DiskError> {
    let query = format!(
        "ASSOCIATORS OF {{Win32_DiskDrive.DeviceID='{}'}} WHERE AssocClass=Win32_DiskDriveToDiskPartition",
        device_id
    );

    let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(query)?;

    let partitions = results
        .iter()
        .filter_map(|result| process_partition(wmi_con, result, partition_count))
        .collect();

    Ok(partitions)
}

/// Process a single partition
fn process_partition(
    wmi_con: &WMIConnection,
    partition_data: &HashMap<String, Variant>,
    partition_count: &mut usize,
) -> Option<Partition> {
    let device_id = get_string_value(partition_data, "DeviceID")?;
    let logical_disk = get_logical_disk(wmi_con, &device_id).ok()??;

    let name = get_string_value(&logical_disk, "Name")?;
    let file_system = get_string_value(&logical_disk, "FileSystem")?;
    let mount_path = format!("{}\\", get_string_value(&logical_disk, "DeviceID")?);

    let file_system = create_file_system(&file_system, &mount_path)?;
    let total_space = get_u64_value(&logical_disk, "Size")?;
    let available_space = get_u64_value(&logical_disk, "FreeSpace")?;

    let partition = Partition::new(
        *partition_count,
        name,
        file_system,
        total_space,
        available_space,
    );

    *partition_count += 1;
    Some(partition)
}

// Helper functions
pub(super) fn get_string_value(map: &HashMap<String, Variant>, key: &str) -> Option<String> {
    match map.get(key) {
        Some(Variant::String(value)) => Some(value.clone()),
        _ => None,
    }
}

pub(super) fn get_u64_value(map: &HashMap<String, Variant>, key: &str) -> Option<u64> {
    match map.get(key) {
        Some(Variant::UI8(value)) => Some(*value),
        _ => None,
    }
}

pub(super) fn get_bool_value(map: &HashMap<String, Variant>, key: &str) -> Option<bool> {
    match map.get(key) {
        Some(Variant::Bool(value)) => Some(*value),
        _ => None,
    }
}

pub(super) fn create_file_system(fs_type: &str, mount_path: &str) -> Option<FileSystem> {
    let mount_path = mount_path.into();
    match SupportedFileSystem::from(fs_type) {
        SupportedFileSystem::NTFS => Some(FileSystem::NTFS(mount_path)),
        SupportedFileSystem::FAT32 => Some(FileSystem::FAT32(mount_path)),
        SupportedFileSystem::EXFAT => Some(FileSystem::EXFAT(mount_path)),
        SupportedFileSystem::NotImplemented(fs) => Some(FileSystem::NotImplemented(fs, mount_path)),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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
