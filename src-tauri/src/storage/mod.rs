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

        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to remove old file: {}", filename))?;
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load() {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path().to_path_buf());

        let data = vec!["hello".to_string(), "world".to_string()];
        storage.save("test.json", &data).unwrap();

        let loaded: Vec<String> = storage.load("test.json").unwrap();
        assert_eq!(loaded, data);
    }

    #[test]
    fn test_load_nonexistent() {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path().to_path_buf());

        let result: Result<Vec<String>> = storage.load("nonexistent.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_or_default_nonexistent() {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path().to_path_buf());

        let loaded: Vec<String> = storage.load_or_default("nonexistent.json");
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_load_or_default_corrupted() {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path().to_path_buf());

        std::fs::write(dir.path().join("corrupted.json"), "not valid json{{{").unwrap();
        let loaded: Vec<String> = storage.load_or_default("corrupted.json");
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_exists() {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path().to_path_buf());

        assert!(!storage.exists("test.json"));

        storage.save("test.json", &vec![1, 2, 3]).unwrap();
        assert!(storage.exists("test.json"));
    }

    #[test]
    fn test_save_overwrites() {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path().to_path_buf());

        storage.save("test.json", &vec![1]).unwrap();
        storage.save("test.json", &vec![2]).unwrap();

        let loaded: Vec<i32> = storage.load("test.json").unwrap();
        assert_eq!(loaded, vec![2]);
    }

    #[test]
    fn test_save_struct() {
        let dir = TempDir::new().unwrap();
        let storage = Storage::new(dir.path().to_path_buf());

        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        struct TestStruct {
            name: String,
            value: i32,
        }

        let data = TestStruct { name: "test".to_string(), value: 42 };
        storage.save("struct.json", &data).unwrap();

        let loaded: TestStruct = storage.load("struct.json").unwrap();
        assert_eq!(loaded, data);
    }

    #[test]
    fn test_storage_creates_data_dir() {
        let dir = TempDir::new().unwrap();
        let data_dir = dir.path().join("nested").join("data");
        assert!(!data_dir.exists());

        let _storage = Storage::new(data_dir.clone());
        assert!(data_dir.exists());
    }
}
