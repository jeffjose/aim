use crate::{device::device_info, error::AdbError, library::adb, types::DeviceDetails};
use log::debug;
use std::path::PathBuf;

pub struct CopyArgs {
    pub src: Vec<PathBuf>,
    pub dst: PathBuf,
}

pub async fn run(
    args: CopyArgs,
    devices: &[DeviceDetails],
    host: &str,
    port: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (dst_device_id, dst_path) = parse_device_path(&args.dst)?;
    let has_multiple_sources = args.src.len() > 1;

    let dst_device = if let Some(id) = dst_device_id {
        Some(device_info::find_target_device(devices, Some(&id))?)
    } else {
        None
    };

    for src_path in args.src.iter() {
        let (src_device_id, src_path) = parse_device_path(src_path)?;

        let src_device = if let Some(id) = src_device_id {
            Some(device_info::find_target_device(devices, Some(&id))?)
        } else {
            None
        };

        match (&src_device, &dst_device) {
            (Some(_), Some(_)) => {
                return Err(AdbError::InvalidCopyOperation(
                    "Cannot copy between two devices".to_string(),
                )
                .into())
            }
            (None, None) => {
                return Err(AdbError::InvalidCopyOperation(
                    "At least one path must specify a device".to_string(),
                )
                .into())
            }
            (Some(device), None) => {
                debug!("Copying from device {} to local", device.adb_id);
                let adb_id = Some(device.adb_id.as_str());
                adb::pull(host, port, adb_id, &src_path, &dst_path, adb::ProgressDisplay::Show).await?;
            }
            (None, Some(device)) => {
                debug!("Copying from local to device {}", device.adb_id);
                let adb_id = Some(device.adb_id.as_str());
                adb::push(host, port, adb_id, &src_path, &dst_path, has_multiple_sources, adb::ProgressDisplay::Show).await?;
            }
        }
    }

    Ok(())
}

pub fn parse_device_path(
    path: &PathBuf,
) -> Result<(Option<String>, PathBuf), Box<dyn std::error::Error>> {
    let path_str = path.to_string_lossy();
    if let Some(colon_idx) = path_str.find(':') {
        let (device, path) = path_str.split_at(colon_idx);
        Ok((
            Some(device.to_string()),
            PathBuf::from(&path[1..]), // Skip the colon
        ))
    } else {
        Ok((None, path.clone()))
    }
}
