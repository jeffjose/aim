use crate::core::types::{Device, DeviceId, OutputFormat};
use crate::error::Result;

/// Shared context for all commands
#[derive(Debug, Clone)]
pub struct CommandContext {
    #[allow(dead_code)]
    pub device: Option<Device>,
    pub output_format: OutputFormat,
    #[allow(dead_code)]
    pub verbose: bool,
    pub quiet: bool,
}

#[allow(dead_code)]
impl CommandContext {
    pub fn new() -> Self {
        Self {
            device: None,
            output_format: OutputFormat::Table,
            verbose: false,
            quiet: false,
        }
    }
    
    pub fn with_device(mut self, device: Device) -> Self {
        self.device = Some(device);
        self
    }
    
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }
    
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    pub fn with_quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }
    
    /// Get the device ID if a device is selected
    pub fn device_id(&self) -> Option<&DeviceId> {
        self.device.as_ref().map(|d| &d.id)
    }
    
    /// Check if a device is selected and available
    pub fn has_available_device(&self) -> bool {
        self.device.as_ref().map_or(false, |d| d.is_available())
    }
    
    /// Get device for commands that require one
    pub fn require_device(&self) -> Result<&Device> {
        self.device.as_ref().ok_or_else(|| {
            crate::error::AimError::DeviceIdRequired
        })
    }
    
    /// Check if progress/status messages should be shown
    /// Returns false if quiet mode is enabled or output format is JSON
    pub fn should_show_progress(&self) -> bool {
        !self.quiet && self.output_format != OutputFormat::Json
    }
}

impl Default for CommandContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for commands that can be executed with context
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait Command {
    type Args;
    type Output;
    
    async fn execute(&self, ctx: &CommandContext, args: Self::Args) -> Result<Self::Output>;
}

/// Builder for creating command contexts
pub struct CommandContextBuilder {
    #[allow(dead_code)]
    ctx: CommandContext,
}

#[allow(dead_code)]
impl CommandContextBuilder {
    pub fn new() -> Self {
        Self {
            ctx: CommandContext::new(),
        }
    }
    
    pub fn device(mut self, device: Device) -> Self {
        self.ctx.device = Some(device);
        self
    }
    
    pub fn output_format(mut self, format: OutputFormat) -> Self {
        self.ctx.output_format = format;
        self
    }
    
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.ctx.verbose = verbose;
        self
    }
    
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.ctx.quiet = quiet;
        self
    }
    
    pub fn build(self) -> CommandContext {
        self.ctx
    }
}

impl Default for CommandContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}