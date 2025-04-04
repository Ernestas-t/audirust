use std::fs;
use std::path::{Path, PathBuf};

pub struct FileManager {
    // Current directory path
    pub current_dir: PathBuf,
    // List of files and directories in the current directory
    pub entries: Vec<PathBuf>,
    // Currently selected file index
    pub selected_index: usize,
}

impl FileManager {
    pub fn new() -> Self {
        // Start with the current directory
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let mut manager = Self {
            current_dir,
            entries: Vec::new(),
            selected_index: 0,
        };

        // Scan for files and directories
        manager.refresh_files();

        manager
    }

    pub fn refresh_files(&mut self) {
        // Clear current list
        self.entries.clear();

        // Read directory and add all entries
        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                // Include directories and audio files
                if path.is_dir() || self.is_audio_file(&path) {
                    self.entries.push(path);
                }
            }
        }

        // Sort directories first, then files
        self.entries.sort_by(|a, b| {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();

            if a_is_dir && !b_is_dir {
                std::cmp::Ordering::Less
            } else if !a_is_dir && b_is_dir {
                std::cmp::Ordering::Greater
            } else {
                // Both are files or both are directories, sort by name
                a.file_name().cmp(&b.file_name())
            }
        });

        // Reset selection if needed
        if !self.entries.is_empty() && self.selected_index >= self.entries.len() {
            self.selected_index = 0;
        }
    }

    pub fn is_audio_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            return matches!(ext.as_str(), "wav" | "mp3" | "ogg" | "flac");
        }
        false
    }

    pub fn select_next(&mut self) {
        if !self.entries.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.entries.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.entries.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.entries.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn get_selected_file(&self) -> Option<PathBuf> {
        if self.entries.is_empty() {
            None
        } else {
            Some(self.entries[self.selected_index].clone())
        }
    }

    pub fn change_directory(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.current_dir = path;
            self.refresh_files();
            self.selected_index = 0; // Reset selection when changing directory
        }
    }

    pub fn go_to_parent_dir(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.refresh_files();
            self.selected_index = 0; // Reset selection when changing directory
        }
    }
}
