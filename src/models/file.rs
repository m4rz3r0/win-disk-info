use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq)]
pub struct FileEntry {
    path: PathBuf,
    size: usize,
    magic_bytes: Vec<u8>,
    hash: Option<String>,
    suspicious: bool,
}

impl FileEntry {
    pub fn name(&self) -> &OsStr {
        self.path
            .file_name()
            .unwrap_or_else(|| self.path.as_os_str())
    }

    pub fn extension(&self) -> Option<&OsStr> {
        self.path.extension()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn into_path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn magic_bytes(&self) -> &[u8] {
        &self.magic_bytes
    }

    pub fn set_magic_bytes(&mut self, bytes: &[u8]) {
        self.magic_bytes = bytes.to_vec();
    }

    pub fn hash(&self) -> Option<&String> {
        self.hash.as_ref()
    }

    pub fn suspicious(&self) -> bool {
        self.suspicious
    }

    pub fn set_suspicious(&mut self) {
        self.suspicious = true;
    }

    pub fn calculate_hash(&mut self) {
        self.hash = sha256::try_digest(self.path())
            .ok()
            .map(|hash| hash.to_string());
    }
}
