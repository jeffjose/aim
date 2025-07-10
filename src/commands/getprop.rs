use crate::commands::SubCommand;
use crate::core::context::CommandContext;
use crate::cli::OutputType;
use crate::error::Result;
use crate::library::adb::{getprop_async, getprops_parallel};
use async_trait::async_trait;
use colored::*;
use comfy_table::Table;
use std::collections::HashMap;
use crate::utils::print_colored_json;

pub struct GetpropCommand;

#[derive(Debug, Clone, clap::Args)]
pub struct GetpropArgs {
    /// Comma-separated list of property names to query. If empty, all properties will be shown
    #[clap(default_value = "")]
    pub propnames: String,

    /// Device ID (required if multiple devices are connected)
    pub device_id: Option<String>,

    /// Output format
    #[clap(short, long, value_enum, default_value_t = OutputType::Plain)]
    pub output: OutputType,
}

impl GetpropCommand {
    pub fn new() -> Self {
        Self
    }
    
    
    async fn get_properties(
        &self,
        device_id: &str,
        propnames: Vec<String>,
        host: &str,
        port: u16,
    ) -> Result<HashMap<String, String>> {
        let port_str = port.to_string();
        if propnames.is_empty() {
            // Get all properties
            let empty_props: Vec<String> = vec![];
            let props = getprops_parallel(host, &port_str, &empty_props, Some(device_id)).await;
            Ok(props)
        } else {
            // Get specific properties
            let mut props = HashMap::new();
            for prop in propnames {
                let value = getprop_async(host, &port_str, &prop, Some(device_id)).await?;
                props.insert(prop, value);
            }
            Ok(props)
        }
    }
}

#[async_trait]
impl SubCommand for GetpropCommand {
    type Args = GetpropArgs;
    
    async fn run(&self, ctx: &CommandContext, args: Self::Args) -> Result<()> {
        let device = ctx.require_device()?;
        let (host, port) = crate::commands::runner::get_adb_connection_params();
        
        // Parse comma-separated property names
        let propnames: Vec<String> = if args.propnames.is_empty() {
            vec![]
        } else {
            args.propnames.split(',').map(|s| s.trim().to_string()).collect()
        };
        
        let device_id = device.id.to_string();
        let results = self.get_properties(&device_id, propnames.clone(), host, port).await?;
        
        match args.output {
            OutputType::Plain => {
                // For single property, just print value
                if propnames.len() == 1 {
                    if let Some(value) = results.get(&propnames[0]) {
                        println!("{}", value.trim().bright_white());
                    }
                } else {
                    // For multiple or all properties, print property=value format
                    let mut sorted_props: Vec<_> = results.iter().collect();
                    sorted_props.sort_by(|a, b| a.0.cmp(b.0));
                    
                    for (propname, value) in sorted_props {
                        println!("{}={}", propname.cyan(), value.trim().bright_white());
                    }
                }
            }
            OutputType::Json => {
                print_colored_json(&results)?;
            }
            OutputType::Table => {
                let mut table = Table::new();
                table.set_header(vec!["Property", "Value"]);
                table.load_preset(comfy_table::presets::NOTHING);
                
                let mut sorted_props: Vec<_> = results.iter().collect();
                sorted_props.sort_by(|a, b| a.0.cmp(b.0));
                
                for (propname, value) in sorted_props {
                    table.add_row(vec![propname, value.trim()]);
                }
                
                println!("{table}");
            }
        }
        
        Ok(())
    }
}