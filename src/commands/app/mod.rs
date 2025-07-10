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
    /// Backup app data
    Backup(backup::BackupArgs),
    
    /// Clear app data
    Clear(clear::ClearArgs),
    
    /// Show detailed information about an app
    Info(info::InfoArgs),
    
    /// List installed applications
    #[command(alias = "ls")]
    List(list::ListArgs),
    
    /// List app permissions
    Permissions(permissions::PermissionsArgs),
    
    /// Pull APK from device
    Pull(pull::PullArgs),
    
    /// Restore app data from backup
    Restore(restore::RestoreArgs),
    
    /// Start an app
    Start(start::StartArgs),
    
    /// Force stop an app
    Stop(stop::StopArgs),
    
    /// Uninstall an app
    Uninstall(uninstall::UninstallArgs),
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
        AppCommands::Info(args) => {
            let cmd = InfoCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::List(args) => {
            let cmd = ListCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Permissions(args) => {
            let cmd = PermissionsCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Pull(args) => {
            let cmd = PullCommand::new();
            cmd.run(ctx, args).await
        }
        AppCommands::Restore(args) => {
            let cmd = RestoreCommand::new();
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
        AppCommands::Uninstall(args) => {
            let cmd = UninstallCommand::new();
            cmd.run(ctx, args).await
        }
    }
}