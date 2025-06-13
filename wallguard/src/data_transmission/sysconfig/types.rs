use std::path::PathBuf;

#[derive(Debug)]
pub struct FileData {
    pub filename: String,
    pub content: Vec<u8>,
}

#[derive(Debug)]
pub struct FileInfo {
    pub path: PathBuf,
    pub mtime: u128,
}

pub type Snapshot = Vec<FileData>;
