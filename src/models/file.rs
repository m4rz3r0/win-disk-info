use std::path::Path;
use std::{ffi::OsStr, path::PathBuf};
use chrono::{DateTime, Local};
use walkdir::DirEntry;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub modified: DateTime<Local>,
    pub is_hidden: bool,
    pub extension: Option<String>,
}

impl From<DirEntry> for FileEntry {
    fn from(entry: DirEntry) -> Self {
        let path = entry.path().to_path_buf();
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = entry.metadata().unwrap_or_else(|_| panic!("No se pudo obtener metadata para {}", path.display()));
        
        let size = metadata.len();
        
        // Convertir SystemTime a DateTime<Local>
        let modified = metadata.modified()
            .map(|time| DateTime::<Local>::from(time))
            .unwrap_or_else(|_| Local::now());
        
        let is_hidden = name.starts_with('.');
        let extension = path.extension()
            .map(|ext| ext.to_string_lossy().to_string());
        
        FileEntry {
            path,
            name,
            size,
            modified,
            is_hidden,
            extension,
        }
    }
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

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn modified(&self) -> DateTime<Local> {
        self.modified
    }
}
