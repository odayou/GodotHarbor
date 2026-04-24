use std::fs;
use std::path::PathBuf;
use serde::de::DeserializeOwned;
use serde::Serialize;
use anyhow::{Result, Context};

pub struct Storage {
    data_dir: PathBuf,
}

impl Storage {
    pub fn new(data_dir: PathBuf) -> Self {
        fs::create_dir_all(&data_dir).ok();
        Self { data_dir }
    }

    pub fn load<T: DeserializeOwned>(&self, filename: &str) -> Result<T> {
        let path = self.data_dir.join(filename);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file: {}", filename))?;
        let data = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON from: {}", filename))?;
        Ok(data)
    }

    pub fn save<T: Serialize>(&self, filename: &str, data: &T) -> Result<()> {
        let path = self.data_dir.join(filename);
        let content = serde_json::to_string_pretty(data)
            .with_context(|| format!("Failed to serialize data for: {}", filename))?;

        let temp_path = path.with_extension("tmp");

        fs::write(&temp_path, &content)
            .with_context(|| format!("Failed to write temp file: {}.tmp", filename))?;

        fs::rename(&temp_path, &path)
            .with_context(|| format!("Failed to rename temp file to: {}", filename))?;

        Ok(())
    }

    pub fn exists(&self, filename: &str) -> bool {
        self.data_dir.join(filename).exists()
    }

    pub fn load_or_default<T: DeserializeOwned + Default>(&self, filename: &str) -> T {
        if self.exists(filename) {
            match self.load(filename) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Warning: failed to load {}: {}, using default", filename, e);
                    T::default()
                }
            }
        } else {
            T::default()
        }
    }
}
