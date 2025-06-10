use crate::constants::DUMP_DIR;
use nullnet_libwallguard::{PacketsData, SystemResourcesData};
use std::ops::RangeTo;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use tokio::fs;

#[derive(Clone, Debug)]
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
            .expect("Failed to read packets and resources dumps directory");
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
            .expect("Failed to read packets and resources dumps directory");
        while let Ok(Some(file)) = dir.next_entry().await {
            if let Ok(meta) = file.metadata().await {
                size += meta.size();
            }
        }
        size >= self.max_size
    }

    fn get_file_path(&self, time: &str, dump_item: &DumpItem) -> String {
        match dump_item {
            DumpItem::Packets(_) => format!("{}/{time}_packets", self.path),
            DumpItem::Resources(_) => format!("{}/{time}_resources", self.path),
            DumpItem::Empty => format!("{}/{time}_empty", self.path),
        }
    }

    pub(crate) async fn dump_item_to_file(&self, dump_item: DumpItem) {
        let now = chrono::Utc::now().to_rfc3339();
        let file_path = self.get_file_path(&now, &dump_item);
        tokio::fs::write(
            file_path,
            serde_json::to_string(&dump_item).expect("Failed to serialize item"),
        )
        .await
        .expect("Failed to write dump file");
    }

    pub(crate) async fn update_items_dump_file(&self, file_path: PathBuf, mut dump: DumpItem) {
        dump.set_token(String::new());
        tokio::fs::write(
            file_path,
            serde_json::to_string(&dump).expect("Failed to serialize items"),
        )
        .await
        .expect("Failed to write dump file");
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
#[serde(untagged)]
pub(crate) enum DumpItem {
    Packets(PacketsData),
    Resources(SystemResourcesData),
    #[default]
    Empty,
}

impl DumpItem {
    pub(crate) fn set_token(&mut self, token: String) {
        match self {
            DumpItem::Packets(packets) => packets.token = token,
            DumpItem::Resources(resources) => resources.token = token,
            DumpItem::Empty => {}
        }
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            DumpItem::Packets(packets) => packets.packets.len(),
            DumpItem::Resources(resources) => resources.resources.len(),
            DumpItem::Empty => 0,
        }
    }

    pub(crate) fn drain(&mut self, range: RangeTo<usize>) {
        match self {
            DumpItem::Packets(packets) => {
                packets.packets.drain(range);
            }
            DumpItem::Resources(resources) => {
                resources.resources.drain(range);
            }
            DumpItem::Empty => {}
        }
    }
}
