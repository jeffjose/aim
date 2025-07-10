use clap::Subcommand;
use crate::error::Result;
use crate::core::context::CommandContext;
use crate::commands::SubCommand;

mod list;
mod clear;
mod pull;
mod backup;
mod stop;
mod start;

pub use list::ListCommand;
pub use clear::ClearCommand;
pub use pull::PullCommand;
pub use backup::BackupCommand;
pub use stop::StopCommand;
pub use start::StartCommand;

#[derive(Debug, Clone, Subcommand)]
pub enum AppCommands {
    /// Backup app data
    Backup(backup::BackupArgs),
    
    /// Clear app data
    Clear(clear::ClearArgs),
    
    /// List installed applications
    #[command(alias = "ls")]
    List(list::ListArgs),
    
    /// Pull APK from device
    Pull(pull::PullArgs),
    
    /// Start an app
    Start(start::StartArgs),
    
    /// Force stop an app
    Stop(stop::StopArgs),
}

impl AppCommands {
    /// Get the device_id from any app subcommand
    pub fn device_id(&self) -> Option<&str> {
        match self {
            AppCommands::Backup(args) => args.device_id.as_deref(),
            AppCommands::Clear(args) => args.device_id.as_deref(),
            AppCommands::List(args) => args.device_id.as_deref(),
            AppCommands::Pull(args) => args.device_id.as_deref(),
            AppCommands::Start(args) => args.device_id.as_deref(),
            AppCommands::Stop(args) => args.device_id.as_deref(),
        }
    }
}

pub async fn run(ctx: &CommandContext, cmd: AppCommands) -> Result<()> {
    match cmd {
        AppCommands::Backup(args) => {
            let cmd = BackupCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Clear(args) => {
            let cmd = ClearCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::List(args) => {
            let cmd = ListCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Pull(args) => {
            let cmd = PullCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Start(args) => {
            let cmd = StartCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Stop(args) => {
            let cmd = StopCommand::new();
            cmd.run(ctx, args).await
        }
    }
}