use crate::output::{TableFormat, PlainFormat};
use std::path::Path;

/// File information for display
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub permissions: String,
    pub file_type: String,
    pub modified: Option<String>,
}

impl FileInfo {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size: 0,
            permissions: String::new(),
            file_type: "file".to_string(),
            modified: None,
        }
    }
    
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }
    
    pub fn with_permissions(mut self, permissions: impl Into<String>) -> Self {
        self.permissions = permissions.into();
        self
    }
    
    pub fn with_type(mut self, file_type: impl Into<String>) -> Self {
        self.file_type = file_type.into();
        self
    }
    
    pub fn with_modified(mut self, modified: impl Into<String>) -> Self {
        self.modified = Some(modified.into());
        self
    }
    
    /// Format size in human-readable format
    pub fn format_size(&self) -> String {
        const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
        
        if self.size == 0 {
            return "0B".to_string();
        }
        
        let mut size = self.size as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{}{}", self.size, UNITS[unit_index])
        } else {
            format!("{:.1}{}", size, UNITS[unit_index])
        }
    }
}

/// Formatter for file listings
pub struct FileFormatter;

impl TableFormat for FileInfo {
    fn headers() -> Vec<&'static str> {
        vec!["NAME", "SIZE", "TYPE", "PERMISSIONS", "MODIFIED"]
    }
    
    fn row(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.format_size(),
            self.file_type.clone(),
            self.permissions.clone(),
            self.modified.clone().unwrap_or_default(),
        ]
    }
}

impl PlainFormat for FileInfo {
    fn plain(&self) -> String {
        format!("{} {} {} {}", 
            self.permissions,
            self.format_size(),
            self.modified.as_ref().unwrap_or(&"-".to_string()),
            self.name
        )
    }
}

/// Transfer progress information
#[derive(Debug, Clone, serde::Serialize)]
pub struct TransferInfo {
    pub file_name: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub rate: Option<f64>, // bytes per second
}

impl TransferInfo {
    pub fn new(file_name: impl Into<String>, total_bytes: u64) -> Self {
        Self {
            file_name: file_name.into(),
            bytes_transferred: 0,
            total_bytes,
            rate: None,
        }
    }
    
    pub fn update(&mut self, bytes_transferred: u64) {
        self.bytes_transferred = bytes_transferred;
    }
    
    pub fn with_rate(mut self, rate: f64) -> Self {
        self.rate = Some(rate);
        self
    }
    
    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.bytes_transferred as f64 / self.total_bytes as f64) * 100.0
        }
    }
    
    pub fn format_rate(&self) -> String {
        if let Some(rate) = self.rate {
            let file_info = FileInfo::new("").with_size(rate as u64);
            format!("{}/s", file_info.format_size())
        } else {
            "-".to_string()
        }
    }
}

impl TableFormat for TransferInfo {
    fn headers() -> Vec<&'static str> {
        vec!["FILE", "PROGRESS", "RATE"]
    }
    
    fn row(&self) -> Vec<String> {
        let progress = format!("{:.1}%", self.percentage());
        vec![
            self.file_name.clone(),
            progress,
            self.format_rate(),
        ]
    }
}

impl PlainFormat for TransferInfo {
    fn plain(&self) -> String {
        format!("{}: {:.1}% ({})", 
            self.file_name,
            self.percentage(),
            self.format_rate()
        )
    }
}