use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub action: String,
    pub target: String,
    pub detail: String,
}

pub struct OperationLogger {
    log_dir: PathBuf,
}

impl OperationLogger {
    pub fn new(data_dir: PathBuf) -> Self {
        let log_dir = data_dir.join("logs");
        fs::create_dir_all(&log_dir).ok();
        Self { log_dir }
    }

    fn write_entry(&self, entry: &LogEntry) -> Result<()> {
        let date = entry.timestamp.format("%Y-%m-%d").to_string();
        let log_file = self.log_dir.join(format!("{}.jsonl", date));

        let mut line = serde_json::to_string(entry)?;
        line.push('\n');

        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;
        file.write_all(line.as_bytes())?;

        Ok(())
    }

    pub fn log(&self, action: &str, target: &str, detail: &str) -> Result<()> {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: "success".to_string(),
            action: action.to_string(),
            target: target.to_string(),
            detail: detail.to_string(),
        };
        self.write_entry(&entry)
    }

    pub fn log_error(&self, action: &str, target: &str, error: &str) -> Result<()> {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: "error".to_string(),
            action: action.to_string(),
            target: target.to_string(),
            detail: error.to_string(),
        };
        self.write_entry(&entry)
    }

    pub fn get_logs(&self, limit: usize) -> Result<Vec<LogEntry>> {
        let mut entries = Vec::new();

        if !self.log_dir.exists() {
            return Ok(entries);
        }

        let mut files: Vec<_> = fs::read_dir(&self.log_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().map(|ext| ext == "jsonl").unwrap_or(false)
            })
            .collect();

        files.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

        for file in files {
            let content = fs::read_to_string(file.path())?;
            for line in content.lines().rev() {
                if let Ok(entry) = serde_json::from_str::<LogEntry>(line) {
                    entries.push(entry);
                    if entries.len() >= limit {
                        entries.reverse();
                        return Ok(entries);
                    }
                }
            }
        }

        entries.reverse();
        Ok(entries)
    }
}
