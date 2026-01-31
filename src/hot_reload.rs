#![allow(clippy::type_complexity)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, UNIX_EPOCH};

#[derive(Clone)]
pub struct FileWatcher {
    files: Arc<Mutex<HashMap<PathBuf, u64>>>,
    callbacks: Arc<Mutex<HashMap<PathBuf, Box<dyn FnMut() + Send>>>>,
}

impl FileWatcher {
    pub fn new() -> Self {
        FileWatcher {
            files: Arc::new(Mutex::new(HashMap::new())),
            callbacks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn watch_file<F>(&self, path: PathBuf, callback: F)
    where
        F: FnMut() + Send + 'static,
    {
        let modified = Self::get_modified_time(&path).unwrap_or(0);

        {
            let mut files = self.files.lock().unwrap();
            files.insert(path.clone(), modified);
        }

        {
            let mut callbacks = self.callbacks.lock().unwrap();
            callbacks.insert(path, Box::new(callback));
        }
    }

    pub fn start_watching(&self) {
        let files = Arc::clone(&self.files);
        let callbacks = Arc::clone(&self.callbacks);

        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(500));

            let changed_files: Vec<PathBuf> = {
                let mut files_guard = files.lock().unwrap();
                let mut changed = Vec::new();

                for (path, old_time) in files_guard.iter() {
                    if let Ok(new_time) = Self::get_modified_time(path) {
                        if new_time > *old_time {
                            changed.push(path.clone());
                        }
                    }
                }

                for path in &changed {
                    if let Ok(new_time) = Self::get_modified_time(path) {
                        files_guard.insert(path.clone(), new_time);
                    }
                }

                changed
            };

            for changed_file in changed_files {
                let mut callbacks_guard = callbacks.lock().unwrap();
                if let Some(callback) = callbacks_guard.get_mut(&changed_file) {
                    callback();
                }
            }
        });
    }

    fn get_modified_time(path: &Path) -> Result<u64, String> {
        let metadata = fs::metadata(path).map_err(|e| format!("Failed to get metadata: {}", e))?;

        let modified = metadata
            .modified()
            .map_err(|e| format!("Failed to get modified time: {}", e))?;

        let duration = modified
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to convert time: {}", e))?;

        Ok(duration.as_secs())
    }
}

pub struct HotReloader {
    watcher: FileWatcher,
    reload_queue: Arc<Mutex<Vec<PathBuf>>>,
}

impl HotReloader {
    pub fn new() -> Self {
        HotReloader {
            watcher: FileWatcher::new(),
            reload_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn watch_script(&self, path: PathBuf) {
        let reload_queue = Arc::clone(&self.reload_queue);
        let path_clone = path.clone();

        self.watcher.watch_file(path, move || {
            println!("File changed: {:?}, queueing for reload...", path_clone);
            let mut queue = reload_queue.lock().unwrap();
            queue.push(path_clone.clone());
        });
    }

    pub fn start(&self) {
        self.watcher.start_watching();
    }

    pub fn get_pending_reloads(&self) -> Vec<PathBuf> {
        let mut queue = self.reload_queue.lock().unwrap();
        let pending = queue.clone();
        queue.clear();
        pending
    }

    pub fn has_pending_reloads(&self) -> bool {
        let queue = self.reload_queue.lock().unwrap();
        !queue.is_empty()
    }
}

pub struct CodeReloader {
    source_cache: HashMap<PathBuf, String>,
    hot_reloader: HotReloader,
}

impl CodeReloader {
    pub fn new() -> Self {
        CodeReloader {
            source_cache: HashMap::new(),
            hot_reloader: HotReloader::new(),
        }
    }

    pub fn enable_hot_reload(&mut self, path: PathBuf) -> Result<(), String> {
        let source =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;

        self.source_cache.insert(path.clone(), source);
        self.hot_reloader.watch_script(path);

        Ok(())
    }

    pub fn start_watching(&self) {
        self.hot_reloader.start();
    }

    pub fn check_and_reload(&mut self) -> Vec<(PathBuf, String)> {
        let mut reloaded = Vec::new();
        let pending = self.hot_reloader.get_pending_reloads();

        for path in pending {
            if let Ok(new_source) = fs::read_to_string(&path) {
                self.source_cache.insert(path.clone(), new_source.clone());
                reloaded.push((path, new_source));
            }
        }

        reloaded
    }

    pub fn get_cached_source(&self, path: &Path) -> Option<&String> {
        self.source_cache.get(path)
    }
}

#[derive(Debug, Clone)]
pub struct ReloadContext {
    pub preserved_state: HashMap<String, String>,
    pub reload_count: usize,
}

impl ReloadContext {
    pub fn new() -> Self {
        ReloadContext {
            preserved_state: HashMap::new(),
            reload_count: 0,
        }
    }

    pub fn preserve_variable(&mut self, name: String, value: String) {
        self.preserved_state.insert(name, value);
    }

    pub fn restore_variable(&self, name: &str) -> Option<&String> {
        self.preserved_state.get(name)
    }

    pub fn increment_reload(&mut self) {
        self.reload_count += 1;
    }

    pub fn get_reload_count(&self) -> usize {
        self.reload_count
    }
}

pub fn enable_hot_reload_for_directory(dir: &Path) -> Result<CodeReloader, String> {
    let mut reloader = CodeReloader::new();

    let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("talon") {
            reloader.enable_hot_reload(path)?;
        }
    }

    reloader.start_watching();
    Ok(reloader)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_file_watcher() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.talon");

        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "let x = 1").unwrap();

        let watcher = FileWatcher::new();
        let changed = Arc::new(Mutex::new(false));
        let changed_clone = Arc::clone(&changed);

        watcher.watch_file(file_path.clone(), move || {
            let mut changed = changed_clone.lock().unwrap();
            *changed = true;
        });

        watcher.start_watching();
        thread::sleep(Duration::from_secs(1));

        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "let x = 2").unwrap();

        thread::sleep(Duration::from_secs(2));

        let is_changed = *changed.lock().unwrap();
        assert!(is_changed);
    }
}
