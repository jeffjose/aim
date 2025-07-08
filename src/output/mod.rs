use crate::error::Result;
use comfy_table::Table;
use colored::*;
use serde::Serialize;

/// Unified output formatter for all commands
pub struct OutputFormatter {
    color_enabled: bool,
    quiet: bool,
}

impl OutputFormatter {
    pub fn new() -> Self {
        Self {
            color_enabled: true,
            quiet: false,
        }
    }
    
    pub fn with_color(mut self, enabled: bool) -> Self {
        self.color_enabled = enabled;
        self
    }
    
    pub fn with_quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }
    
    /// Format items as a table
    pub fn table<T: TableFormat>(&self, items: &[T]) -> Result<()> {
        if self.quiet {
            return Ok(());
        }
        
        let mut table = Table::new();
        table.set_header(T::headers());
        table.load_preset(comfy_table::presets::NOTHING);
        
        for item in items {
            table.add_row(item.row());
        }
        
        println!("{}", table);
        Ok(())
    }
    
    /// Format items as JSON
    pub fn json<T: Serialize>(&self, items: &T) -> Result<()> {
        if self.quiet {
            return Ok(());
        }
        
        if self.color_enabled {
            crate::utils::print_colored_json(items)?;
        } else {
            let json = serde_json::to_string_pretty(items)?;
            println!("{}", json);
        }
        Ok(())
    }
    
    /// Format items as plain text
    pub fn plain<T: PlainFormat>(&self, items: &[T]) -> Result<()> {
        if self.quiet {
            return Ok(());
        }
        
        for item in items {
            println!("{}", item.plain());
        }
        Ok(())
    }
    
    /// Print a message (respecting quiet mode)
    pub fn message(&self, msg: &str) -> Result<()> {
        if !self.quiet {
            println!("{}", msg);
        }
        Ok(())
    }
    
    /// Print an info message
    pub fn info(&self, msg: &str) -> Result<()> {
        if !self.quiet {
            if self.color_enabled {
                println!("{}", msg.bright_blue());
            } else {
                println!("INFO: {}", msg);
            }
        }
        Ok(())
    }
    
    /// Print a success message
    pub fn success(&self, msg: &str) -> Result<()> {
        if !self.quiet {
            if self.color_enabled {
                println!("{}", msg.bright_green());
            } else {
                println!("SUCCESS: {}", msg);
            }
        }
        Ok(())
    }
    
    /// Print a warning message
    pub fn warning(&self, msg: &str) -> Result<()> {
        if !self.quiet {
            if self.color_enabled {
                eprintln!("{}", msg.bright_yellow());
            } else {
                eprintln!("WARNING: {}", msg);
            }
        }
        Ok(())
    }
    
    /// Print an error message
    pub fn error(&self, msg: &str) -> Result<()> {
        if self.color_enabled {
            eprintln!("{}", msg.bright_red());
        } else {
            eprintln!("ERROR: {}", msg);
        }
        Ok(())
    }
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can be formatted as a table
pub trait TableFormat {
    fn headers() -> Vec<&'static str>;
    fn row(&self) -> Vec<String>;
}

/// Trait for types that can be formatted as plain text
pub trait PlainFormat {
    fn plain(&self) -> String;
}

/// Implementation for common types
impl<T: ToString> PlainFormat for T {
    fn plain(&self) -> String {
        self.to_string()
    }
}

/// Module for specific formatters
pub mod device;
pub mod property;
pub mod file;

// Re-exports
pub use device::DeviceFormatter;
pub use property::PropertyFormatter;
pub use file::FileFormatter;