use std::collections::HashMap;
use tokio::io;
use tokio::process::Command;

pub(super) async fn snapshot_processes() -> io::Result<HashMap<u32, String>> {
    let mut map = HashMap::<u32, String>::new();

    let output = Command::new("ps").args(&["aux"]).output().await?;

    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "ps command failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 11 {
            if let Ok(pid) = parts[1].parse::<u32>() {
                let command = parts[10..].join(" ");
                map.insert(pid, command);
            }
        }
    }

    Ok(map)
}
