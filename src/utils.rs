use std::path::Path;
use anyhow::Result;

pub fn ensure_directory_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn sanitize_filename(filename: &str) -> String {
    filename.replace(|c: char| !c.is_ascii_alphanumeric(), "_")
} 