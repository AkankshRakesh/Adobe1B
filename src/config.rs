use std::path::{Path, PathBuf};
use anyhow::Result;

pub struct Config {
    pub collections_dir: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        let collections_dir = current_dir.join("collections");
        Ok(Self { collections_dir })
    }

    pub fn get_collection_paths(&self) -> Result<Vec<(String, PathBuf, PathBuf)>> {
        let mut collections = Vec::new();
        for entry in std::fs::read_dir(&self.collections_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                let input_path = entry.path().join("challenge1b_input.json");
                let output_path = entry.path().join("challenge1b_output.json");
                collections.push((name, input_path, output_path));
            }
        }
        Ok(collections)
    }
}