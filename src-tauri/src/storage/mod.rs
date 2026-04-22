use std::fs;
use std::path::PathBuf;
use serde::de::DeserializeOwned;
use serde::Serialize;
use anyhow::Result;

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
        let content = fs::read_to_string(path)?;
        let data = serde_json::from_str(&content)?;
        Ok(data)
    }

    pub fn save<T: Serialize>(&self, filename: &str, data: &T) -> Result<()> {
        let path = self.data_dir.join(filename);
        let content = serde_json::to_string_pretty(data)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn exists(&self, filename: &str) -> bool {
        self.data_dir.join(filename).exists()
    }

    pub fn load_or_default<T: DeserializeOwned + Default>(&self, filename: &str) -> T {
        if self.exists(filename) {
            self.load(filename).unwrap_or_default()
        } else {
            T::default()
        }
    }
}
