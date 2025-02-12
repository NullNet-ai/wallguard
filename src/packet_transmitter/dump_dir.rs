use crate::constants::DUMP_DIR;
use crate::logger::Logger;
use libwallguard::{Packet, Packets};
use log::Level;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
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

    fn get_file_path(&self, file_name: &str) -> String {
        format!("{}/{file_name}", self.path)
    }

    pub(crate) async fn dump_packets_to_file(&self, packets: Vec<Packet>, uuid: String) {
        let now = chrono::Utc::now().to_rfc3339();
        let file_path = self.get_file_path(&now);
        Logger::log(
            Level::Warn,
            format!(
                "Queue is full. Dumping {} packets to file '{file_path}'",
                packets.len()
            ),
        );
        let dump = Packets {
            uuid,
            packets,
            auth: None,
        };
        tokio::fs::write(
            file_path,
            bincode::serialize(&dump).expect("Failed to serialize packets"),
        )
        .await
        .expect("Failed to write dump file");
    }

    pub(crate) async fn update_dump_file(&self, file_path: PathBuf, mut dump: Packets) {
        dump.auth = None;
        tokio::fs::write(
            file_path,
            bincode::serialize(&dump).expect("Failed to serialize packets"),
        )
        .await
        .expect("Failed to write dump file");
    }
}
