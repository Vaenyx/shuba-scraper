use anyhow::Result;
use indexmap::IndexMap;

use std::fs;
use std::path::Path;

pub fn remove_path(path_str: &str) -> Result<()> {
    let path = Path::new(path_str);

    if !path.exists() {
        return Ok(());
    }

    let metadata = fs::symlink_metadata(path)?;

    if metadata.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }

    return Ok(());
}

pub fn save_singlefile(chapters: IndexMap<u32, String>, out: &str) -> Result<()> {
    let text = chapters
        .values()
        .map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n\n\n");

    std::fs::write(out, text)?;
    return Ok(());
}

pub fn save_dir(chapters: IndexMap<u32, String>, out: &str) -> Result<()> {
    std::fs::create_dir_all(out)?;
    for (idx, text) in chapters {
        std::fs::write(format!("{}/{}", out, idx), text)?;
    }
    return Ok(());
}
