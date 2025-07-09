use clap::Subcommand;
use crate::error::Result;
use crate::core::context::CommandContext;
use crate::commands::SubCommand;

mod list;
mod info;
mod clear;
mod pull;
mod backup;
mod restore;
mod permissions;
mod stop;
mod start;
mod uninstall;

pub use list::ListCommand;
pub use info::InfoCommand;
pub use clear::ClearCommand;
pub use pull::PullCommand;
pub use backup::BackupCommand;
pub use restore::RestoreCommand;
pub use permissions::PermissionsCommand;
pub use stop::StopCommand;
pub use start::StartCommand;
pub use uninstall::UninstallCommand;

#[derive(Debug, Clone, Subcommand)]
pub enum AppCommands {
    /// List installed applications
    List(list::ListArgs),
    
    /// Show detailed information about an app
    Info(info::InfoArgs),
    
    /// Clear app data
    Clear(clear::ClearArgs),
    
    /// Pull APK from device
    Pull(pull::PullArgs),
    
    /// Backup app data
    Backup(backup::BackupArgs),
    
    /// Restore app data from backup
    Restore(restore::RestoreArgs),
    
    /// List app permissions
    Permissions(permissions::PermissionsArgs),
    
    /// Force stop an app
    Stop(stop::StopArgs),
    
    /// Start an app
    Start(start::StartArgs),
    
    /// Uninstall an app
    Uninstall(uninstall::UninstallArgs),
}

pub async fn run(ctx: &CommandContext, cmd: AppCommands) -> Result<()> {
    match cmd {
        AppCommands::List(args) => {
            let cmd = ListCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Info(args) => {
            let cmd = InfoCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Clear(args) => {
            let cmd = ClearCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Pull(args) => {
            let cmd = PullCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Backup(args) => {
            let cmd = BackupCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Restore(args) => {
            let cmd = RestoreCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Permissions(args) => {
            let cmd = PermissionsCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Stop(args) => {
            let cmd = StopCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Start(args) => {
            let cmd = StartCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Uninstall(args) => {
            let cmd = UninstallCommand::new();
            cmd.run(ctx, args).await
        }
    }
}