#[derive(Debug)]
pub struct FileData {
    pub filename: String,
    pub content: Vec<u8>,
}

pub type Snapshot = Vec<FileData>;
