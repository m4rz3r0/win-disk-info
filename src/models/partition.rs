use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum FileSystem {
    BTRFS(Vec<PathBuf>),
    EXT4(PathBuf),
    NTFS(PathBuf),
    FAT32(PathBuf),
    EXFAT(PathBuf),
    XFS(PathBuf),
    ZFS(PathBuf),
    NotImplemented(String, PathBuf),
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Partition {
    id: usize,
    name: String,
    file_system: FileSystem,
    total_space: u64,
    available_space: u64,
}

impl Partition {
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

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn file_system(&self) -> &FileSystem {
        &self.file_system
    }

    pub fn total_space(&self) -> u64 {
        self.total_space
    }

    pub fn available_space(&self) -> u64 {
        self.available_space
    }
}
