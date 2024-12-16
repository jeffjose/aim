use crate::types::DeviceDetails;
use anyhow::Result;
use log::*;

pub struct CopyArgs {
    pub src: String,
    pub dst: String,
}

#[derive(Debug)]
struct ParsedPath {
    device_id: Option<String>,
    path: String,
}

fn parse_path(input: &str) -> Result<ParsedPath> {
    let parts: Vec<&str> = input.split(':').collect();
    match parts.as_slice() {
        // If there's a colon, treat the first part as device_id
        [device_id, path] => Ok(ParsedPath {
            device_id: Some(device_id.to_string()),
            path: path.to_string(),
        }),
        // If there's no colon, treat the entire string as a local path
        [path] => Ok(ParsedPath {
            device_id: None,
            path: path.to_string(),
        }),
        _ => Err(anyhow::anyhow!("Invalid path format. Use [device_id:]path")),
    }
}

async fn get_matching_device<'a>(
    devices: &'a [DeviceDetails],
    partial_id: &str,
) -> Result<&'a DeviceDetails> {
    let matching_devices: Vec<&DeviceDetails> = devices
        .iter()
        .filter(|device| device.matches_id_prefix(partial_id))
        .collect();

    match matching_devices.len() {
        0 => Err(anyhow::anyhow!(
            "No devices found matching ID '{}'",
            partial_id
        )),
        1 => Ok(matching_devices[0]),
        _ => Err(anyhow::anyhow!(
            "Multiple devices match '{}': {:?}",
            partial_id,
            matching_devices
                .iter()
                .map(|d| d.device_name.clone())
                .collect::<Vec<_>>()
        )),
    }
}

pub async fn run(args: CopyArgs, devices: &[DeviceDetails]) -> Result<()> {
    info!("Copy command with src: {}, dst: {}", args.src, args.dst);

    // Parse source and destination paths
    let src = parse_path(&args.src)?;
    let dst = parse_path(&args.dst)?;

    // Resolve device IDs if present
    let src_device = match src.device_id {
        Some(ref partial_id) => Some(get_matching_device(devices, partial_id).await?),
        None => None,
    };

    let dst_device = match dst.device_id {
        Some(ref partial_id) => Some(get_matching_device(devices, partial_id).await?),
        None => None,
    };

    println!("Copy operation details:");
    println!("Source:");
    println!(
        "  Device: {}",
        src_device
            .map(|d| &d.device_name)
            .unwrap_or(&"local".to_string())
    );
    println!("  Path: {}", src.path);
    println!("Destination:");
    println!(
        "  Device: {}",
        dst_device
            .map(|d| &d.device_name)
            .unwrap_or(&"local".to_string())
    );
    println!("  Path: {}", dst.path);

    // TODO: Implement actual copy logic based on source and destination types

    Ok(())
}
