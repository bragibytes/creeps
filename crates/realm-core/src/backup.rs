use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BACKUPS: usize = 5;

fn backup_stamp() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    millis.to_string()
}

pub fn backup_players_json(data_dir: &Path, store_path: &Path) -> io::Result<()> {
    if !store_path.exists() {
        return Ok(());
    }

    let backup_dir = data_dir.join("backups");
    fs::create_dir_all(&backup_dir)?;

    let dest = backup_dir.join(format!("players-{}.json", backup_stamp()));
    fs::copy(store_path, &dest)?;

    let mut files: Vec<PathBuf> = fs::read_dir(&backup_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("players-") && name.ends_with(".json"))
        })
        .collect();

    files.sort_by(|a, b| b.cmp(a));

    for old in files.into_iter().skip(MAX_BACKUPS) {
        let _ = fs::remove_file(old);
    }

    Ok(())
}