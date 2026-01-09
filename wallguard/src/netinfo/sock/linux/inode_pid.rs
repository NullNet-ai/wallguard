use std::{collections::HashMap, path::PathBuf};
use tokio::fs;

async fn read_proc_comm(pid: u32) -> Option<String> {
    let mut path = PathBuf::from("/proc");
    path.push(pid.to_string());
    path.push("comm");

    match fs::read_to_string(&path).await {
        Ok(mut name) => {
            if let Some('\n') = name.chars().last() {
                name.pop();
            }
            Some(name)
        }
        Err(_) => None,
    }
}

pub(super) async fn build_inode_pid_map() -> HashMap<u64, String> {
    let mut map = HashMap::new();

    let Ok(mut rd) = fs::read_dir("/proc").await else {
        return map;
    };

    while let Ok(Some(entry)) = rd.next_entry().await {
        let pid_str = entry.file_name().into_string().unwrap_or_default();

        if pid_str.chars().any(|c| !c.is_ascii_digit()) {
            continue;
        };

        let Ok(pid) = pid_str.parse::<u32>() else {
            continue;
        };

        let Ok(mut fd) = fs::read_dir(&format!("/proc/{pid}/fd")).await else {
            continue;
        };

        while let Ok(Some(fds)) = fd.next_entry().await {
            let Ok(link) = fs::read_link(fds.path()).await else {
                continue;
            };

            if let Some(inner) = link.to_str().and_then(|s| s.strip_prefix("socket:["))
                && let Some(num) = inner.strip_suffix(']')
                && let Ok(inode) = num.parse::<u64>()
                && let Some(process_name) = read_proc_comm(pid).await
            {
                map.insert(inode, process_name);
            }
        }
    }

    map
}
