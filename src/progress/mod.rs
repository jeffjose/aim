use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::sync::Arc;
use std::time::Duration;

/// Trait for progress reporting
#[allow(dead_code)]
pub trait ProgressReporter: Send + Sync {
    fn start(&self, total: u64);
    fn update(&self, current: u64);
    fn finish(&self);
    fn set_message(&self, msg: &str);
    fn inc(&self, delta: u64);
}

/// Indicatif-based progress reporter
pub struct IndicatifProgress {
    bar: ProgressBar,
}

#[allow(dead_code)]
impl IndicatifProgress {
    /// Create a new progress bar
    pub fn new(total: u64) -> Self {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-")
        );
        
        Self { bar }
    }
    
    /// Create a progress bar with custom template
    pub fn with_template(total: u64, template: &str) -> Self {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::default_bar()
                .template(template)
                .unwrap()
                .progress_chars("#>-")
        );
        
        Self { bar }
    }
    
    /// Create a spinner for indeterminate progress
    pub fn spinner() -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        bar.enable_steady_tick(Duration::from_millis(100));
        
        Self { bar }
    }
}

impl ProgressReporter for IndicatifProgress {
    fn start(&self, total: u64) {
        self.bar.set_length(total);
    }
    
    fn update(&self, current: u64) {
        self.bar.set_position(current);
    }
    
    fn finish(&self) {
        self.bar.finish_with_message("Complete");
    }
    
    fn set_message(&self, msg: &str) {
        self.bar.set_message(msg.to_string());
    }
    
    fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }
}

/// No-op progress reporter for when progress reporting is disabled
pub struct NoOpProgress;

impl ProgressReporter for NoOpProgress {
    fn start(&self, _total: u64) {}
    fn update(&self, _current: u64) {}
    fn finish(&self) {}
    fn set_message(&self, _msg: &str) {}
    fn inc(&self, _delta: u64) {}
}

/// Progress reporter factory
#[allow(dead_code)]
pub struct ProgressFactory {
    enabled: bool,
    multi: Option<Arc<MultiProgress>>,
}

#[allow(dead_code)]
impl ProgressFactory {
    /// Create a new progress factory
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            multi: None,
        }
    }
    
    /// Create a factory with multi-progress support
    pub fn with_multi() -> Self {
        Self {
            enabled: true,
            multi: Some(Arc::new(MultiProgress::new())),
        }
    }
    
    /// Create a progress reporter for file transfer
    pub fn file_transfer(&self, file_name: &str, total_bytes: u64) -> Box<dyn ProgressReporter> {
        if !self.enabled {
            return Box::new(NoOpProgress);
        }
        
        let template = format!(
            "{{spinner:.green}} {} [{{bar:40.cyan/blue}}] {{bytes}}/{{total_bytes}} ({{bytes_per_sec}}, {{eta}})",
            file_name
        );
        
        let progress = IndicatifProgress::with_template(total_bytes, &template);
        
        if let Some(multi) = &self.multi {
            multi.add(progress.bar.clone());
        }
        
        Box::new(progress)
    }
    
    /// Create a progress reporter for command execution
    pub fn command(&self, command: &str) -> Box<dyn ProgressReporter> {
        if !self.enabled {
            return Box::new(NoOpProgress);
        }
        
        let progress = IndicatifProgress::spinner();
        progress.set_message(&format!("Running: {}", command));
        
        if let Some(multi) = &self.multi {
            multi.add(progress.bar.clone());
        }
        
        Box::new(progress)
    }
    
    /// Create a generic progress bar
    pub fn generic(&self, total: u64) -> Box<dyn ProgressReporter> {
        if !self.enabled {
            return Box::new(NoOpProgress);
        }
        
        let progress = IndicatifProgress::new(total);
        
        if let Some(multi) = &self.multi {
            multi.add(progress.bar.clone());
        }
        
        Box::new(progress)
    }
    
    /// Create a custom progress bar
    pub fn custom(&self, total: u64, template: &str) -> Box<dyn ProgressReporter> {
        if !self.enabled {
            return Box::new(NoOpProgress);
        }
        
        let progress = IndicatifProgress::with_template(total, template);
        
        if let Some(multi) = &self.multi {
            multi.add(progress.bar.clone());
        }
        
        Box::new(progress)
    }
}

/// Progress context for commands
#[allow(dead_code)]
pub struct ProgressContext {
    factory: ProgressFactory,
    reporters: Vec<Box<dyn ProgressReporter>>,
}

#[allow(dead_code)]
impl ProgressContext {
    /// Create a new progress context
    pub fn new(enabled: bool) -> Self {
        Self {
            factory: ProgressFactory::new(enabled),
            reporters: Vec::new(),
        }
    }
    
    /// Create a context with multi-progress
    pub fn with_multi() -> Self {
        Self {
            factory: ProgressFactory::with_multi(),
            reporters: Vec::new(),
        }
    }
    
    /// Add a file transfer progress
    pub fn add_file_transfer(&mut self, file_name: &str, total_bytes: u64) -> &dyn ProgressReporter {
        let reporter = self.factory.file_transfer(file_name, total_bytes);
        self.reporters.push(reporter);
        self.reporters.last().unwrap().as_ref()
    }
    
    /// Add a command progress
    pub fn add_command(&mut self, command: &str) -> &dyn ProgressReporter {
        let reporter = self.factory.command(command);
        self.reporters.push(reporter);
        self.reporters.last().unwrap().as_ref()
    }
    
    /// Finish all progress reporters
    pub fn finish_all(&mut self) {
        for reporter in &self.reporters {
            reporter.finish();
        }
        self.reporters.clear();
    }
}

