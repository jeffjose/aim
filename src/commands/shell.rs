use crate::commands::{SubCommand, get_device};
use crate::core::context::CommandContext;
use crate::error::Result;
use crate::library::adb::run_shell_command_async;
use async_trait::async_trait;
use std::io::{self, BufRead, Write};

pub struct ShellCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct ShellArgs {
    /// Command to execute (if empty, starts interactive shell)
    #[clap(trailing_var_arg = true)]
    pub command: Vec<String>,

    /// Device ID (required if multiple devices are connected)
    #[clap(short = 'd', long = "device")]
    pub device_id: Option<String>,
}

impl ShellCommand {
    pub fn new() -> Self {
        Self
    }

    async fn run_interactive(&self, host: &str, port: &str, device_id: &str) -> Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        println!("Interactive shell on device {}. Type 'exit' to quit.", device_id);
        println!();

        loop {
            print!("$ ");
            stdout.flush()?;

            let mut input = String::new();
            if stdin.lock().read_line(&mut input)? == 0 {
                // EOF
                break;
            }

            let cmd = input.trim();
            if cmd.is_empty() {
                continue;
            }
            if cmd == "exit" || cmd == "quit" {
                break;
            }

            match run_shell_command_async(host, port, cmd, Some(device_id)).await {
                Ok(output) => {
                    if !output.is_empty() {
                        print!("{}", output);
                        if !output.ends_with('\n') {
                            println!();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl SubCommand for ShellCommand {
    type Args = ShellArgs;

    async fn run(&self, _ctx: &CommandContext, args: Self::Args) -> Result<()> {
        let device = get_device(args.device_id.as_deref()).await?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        let device_id_str = device.id.to_string();
        let port_str = port.to_string();

        if args.command.is_empty() {
            // Interactive mode
            self.run_interactive(host, &port_str, &device_id_str).await
        } else {
            // Single command mode
            let cmd = args.command.join(" ");
            let output = run_shell_command_async(host, &port_str, &cmd, Some(&device_id_str)).await?;

            if !output.is_empty() {
                print!("{}", output);
                if !output.ends_with('\n') {
                    println!();
                }
            }

            Ok(())
        }
    }
}
