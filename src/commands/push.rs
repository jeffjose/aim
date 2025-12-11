use crate::commands::{SubCommand, get_device};
use crate::core::context::CommandContext;
use crate::error::Result;
use crate::library::adb::{push, ProgressDisplay};
use async_trait::async_trait;
use std::path::PathBuf;

pub struct PushCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct PushArgs {
    /// Local file(s) to push
    #[clap(required = true)]
    pub src: Vec<PathBuf>,

    /// Remote destination path on device
    pub dst: String,

    /// Device ID (required if multiple devices are connected)
    pub device_id: Option<String>,

    /// Recursive push (for directories)
    #[clap(short, long)]
    pub recursive: bool,
}

impl PushCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SubCommand for PushCommand {
    type Args = PushArgs;

    async fn run(&self, _ctx: &CommandContext, args: Self::Args) -> Result<()> {
        let device = get_device(args.device_id.as_deref()).await?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        let device_id_str = device.id.to_string();
        let port_str = port.to_string();

        let has_multiple = args.src.len() > 1;

        for src in &args.src {
            println!("Pushing {} to {}", src.display(), args.dst);

            push(
                host,
                &port_str,
                Some(&device_id_str),
                src,
                &PathBuf::from(&args.dst),
                has_multiple,
                ProgressDisplay::Show,
            )
            .await?;
        }

        Ok(())
    }
}
