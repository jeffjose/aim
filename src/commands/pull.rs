use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::error::Result;
use crate::library::adb::{pull, ProgressDisplay};
use async_trait::async_trait;
use std::path::PathBuf;

pub struct PullCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct PullArgs {
    /// Remote file(s) on device to pull
    #[clap(required = true)]
    pub src: Vec<String>,

    /// Local destination path
    #[clap(default_value = ".")]
    pub dst: PathBuf,

    /// Device ID (required if multiple devices are connected)
    pub device_id: Option<String>,
}

impl PullCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SubCommand for PullCommand {
    type Args = PullArgs;

    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        let device_id_str = device.id.to_string();
        let port_str = port.to_string();

        for src in &args.src {
            println!("Pulling {} to {}", src, args.dst.display());

            pull(
                host,
                &port_str,
                Some(&device_id_str),
                &PathBuf::from(src),
                &args.dst,
                ProgressDisplay::Show,
            )
            .await?;
        }

        Ok(())
    }
}
