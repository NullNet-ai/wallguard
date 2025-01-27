use crate::constants::DUMP_DIR;
use std::os::unix::fs::MetadataExt;
use tokio::fs;

#[derive(Clone)]
pub(crate) struct DumpDir {
    path: &'static str,
    max_size: u64,
}

impl DumpDir {
    pub(crate) async fn new(max_size: u64) -> Self {
        fs::create_dir(DUMP_DIR).await.unwrap_or_default();
        Self {
            path: DUMP_DIR,
            max_size,
        }
    }

    pub(crate) async fn get_files_sorted(&self) -> Vec<fs::DirEntry> {
        let mut dir = fs::read_dir(self.path)
            .await
            .expect("Failed to read packet dumps directory");
        let mut files = Vec::new();
        while let Ok(Some(file)) = dir.next_entry().await {
            files.push(file);
        }
        files.sort_by_key(fs::DirEntry::file_name);
        files
    }

    pub(crate) async fn is_full(&self) -> bool {
        let mut size = 0;
        let mut dir = fs::read_dir(self.path)
            .await
            .expect("Failed to read packet dumps directory");
        while let Ok(Some(file)) = dir.next_entry().await {
            if let Ok(meta) = file.metadata().await {
                size += meta.size();
            }
        }
        size >= self.max_size
    }

    pub(crate) fn get_file_path(&self, file_name: &str) -> String {
        format!("{}/{file_name}", self.path)
    }
}
