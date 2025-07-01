use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub fn add_ssh_key_if_missing(public_key: &str) -> std::io::Result<()> {
    let mut auth_keys_path = dirs::home_dir().unwrap_or(PathBuf::from("/root"));
    auth_keys_path.push(".ssh");
    fs::create_dir_all(&auth_keys_path)?;

    auth_keys_path.push("authorized_keys");

    if auth_keys_path.exists() {
        let file = fs::File::open(&auth_keys_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if line.trim() == public_key.trim() {
                return Ok(());
            }
        }
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&auth_keys_path)?;

    writeln!(file, "{}", public_key.trim())?;
    Ok(())
}

pub fn remove_added_ssh_keys() -> std::io::Result<()> {
    let mut auth_keys_path = dirs::home_dir().unwrap_or(PathBuf::from("/root"));
    auth_keys_path.push(".ssh");
    auth_keys_path.push("authorized_keys");

    if !auth_keys_path.exists() {
        return Ok(());
    }

    let file = fs::File::open(&auth_keys_path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.contains("wallguard-system@nullnet.ai"))
        .collect();

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&auth_keys_path)?;

    for line in lines {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}
