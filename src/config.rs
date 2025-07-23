use std::path::PathBuf;
use anyhow::Result;

pub struct Config {
    pub collections_dir: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self> {
        let collections_dir = std::env::current_dir()?.join("collections");
        Ok(Self { collections_dir })
    }

    pub fn get_collections(&self) -> Result<Vec<String>> {
        let mut collections = Vec::new();
        for entry in std::fs::read_dir(&self.collections_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    collections.push(name.to_string());
                }
            }
        }
        Ok(collections)
    }
}