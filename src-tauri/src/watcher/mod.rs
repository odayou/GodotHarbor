use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config, Event, EventKind};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

struct WatcherState {
    watcher: RecommendedWatcher,
    last_emit: Instant,
    watched_paths: HashSet<PathBuf>,
}

pub struct FsWatcher {
    state: Arc<Mutex<Option<WatcherState>>>,
    debounce_interval: Duration,
}

impl FsWatcher {
    pub fn new(debounce_secs: u64) -> Self {
        Self {
            state: Arc::new(Mutex::new(None)),
            debounce_interval: Duration::from_secs(debounce_secs),
        }
    }

    pub fn start(&self, app: AppHandle, directories: Vec<String>) -> Result<(), String> {
        let (tx, rx) = mpsc::channel::<Event>();

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default(),
        )
        .map_err(|e| format!("创建文件监听器失败: {}", e))?;

        {
            let mut state = self.state.lock().map_err(|e| format!("获取监听状态锁失败: {}", e))?;
            *state = Some(WatcherState {
                watcher,
                last_emit: Instant::now() - self.debounce_interval,
                watched_paths: HashSet::new(),
            });
        }

        let state_clone = self.state.clone();
        let debounce = self.debounce_interval;

        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                let is_relevant = match &event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => true,
                    _ => false,
                };

                if !is_relevant {
                    continue;
                }

                let has_project_godot = event.paths.iter().any(|p| {
                    p.file_name()
                        .map(|f| f == "project.godot")
                        .unwrap_or(false)
                });

                let has_harbor_yml = event.paths.iter().any(|p| {
                    p.file_name()
                        .map(|f| f == ".harbor.yml")
                        .unwrap_or(false)
                });

                let is_in_project_dir = event.paths.iter().any(|p| {
                    p.parent()
                        .map(|dir| dir.join("project.godot").exists())
                        .unwrap_or(false)
                });

                if !has_project_godot && !has_harbor_yml && !is_in_project_dir {
                    continue;
                }

                let should_emit = {
                    let mut state = match state_clone.lock() {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    if let Some(ref mut s) = *state {
                        let now = Instant::now();
                        if now.duration_since(s.last_emit) >= debounce {
                            s.last_emit = now;
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };

                if should_emit {
                    let changed_paths: Vec<String> = event
                        .paths
                        .iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect();
                    let _ = app.emit("project-fs-changed", serde_json::json!({
                        "paths": changed_paths,
                        "kind": format!("{:?}", event.kind),
                    }));
                }
            }
        });

        let mut state = self.state.lock().map_err(|e| format!("获取监听状态锁失败: {}", e))?;
        if let Some(ref mut s) = *state {
            for dir in &directories {
                let path = PathBuf::from(dir);
                if path.exists() {
                    if let Err(e) = s.watcher.watch(&path, RecursiveMode::Recursive) {
                        eprintln!("Failed to watch directory {}: {}", dir, e);
                    } else {
                        s.watched_paths.insert(path);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| format!("获取监听状态锁失败: {}", e))?;
        *state = None;
        Ok(())
    }

    pub fn update_directories(&self, directories: Vec<String>) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| format!("获取监听状态锁失败: {}", e))?;
        if let Some(ref mut s) = *state {
            for path in &s.watched_paths {
                let _ = s.watcher.unwatch(path);
            }
            s.watched_paths.clear();

            for dir in &directories {
                let path = PathBuf::from(dir);
                if path.exists() {
                    if let Err(e) = s.watcher.watch(&path, RecursiveMode::Recursive) {
                        eprintln!("Failed to watch directory {}: {}", dir, e);
                    } else {
                        s.watched_paths.insert(path);
                    }
                }
            }
        }
        Ok(())
    }
}
