use crate::error::{AimError, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use log::*;
use std::path::{Path, PathBuf};

const PLATFORM_TOOLS_URL_LINUX: &str =
    "https://dl.google.com/android/repository/platform-tools-latest-linux.zip";
const PLATFORM_TOOLS_URL_MACOS: &str =
    "https://dl.google.com/android/repository/platform-tools-latest-darwin.zip";

/// Resolve the path to a working `adb` binary.
///
/// Resolution order:
/// 1. `ADB_PATH` env var
/// 2. `adb` on system PATH
/// 3. `~/.aim/platform-tools/adb` (auto-downloaded if missing)
pub async fn resolve_adb_path() -> Result<PathBuf> {
    // 1. Explicit override
    if let Ok(path) = std::env::var("ADB_PATH") {
        let p = PathBuf::from(&path);
        debug!("Using ADB_PATH: {}", p.display());
        return Ok(p);
    }

    // 2. System PATH
    if let Some(path) = find_adb_on_path() {
        debug!("Found adb on PATH: {}", path.display());
        return Ok(path);
    }

    // 3. Managed installation
    let managed = managed_adb_path()?;
    if managed.exists() {
        debug!("Using managed adb: {}", managed.display());
        return Ok(managed);
    }

    // 4. Download
    download_platform_tools().await?;

    if managed.exists() {
        Ok(managed)
    } else {
        Err(AimError::Toolchain(
            "Failed to install platform-tools: adb binary not found after extraction".into(),
        ))
    }
}

/// Synchronous version that only checks existing paths (no download).
/// Used by legacy sync code paths.
pub fn resolve_adb_path_sync() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("ADB_PATH") {
        return Ok(PathBuf::from(path));
    }

    if let Some(path) = find_adb_on_path() {
        return Ok(path);
    }

    let managed = managed_adb_path()?;
    if managed.exists() {
        return Ok(managed);
    }

    Err(AimError::Toolchain(
        "adb not found. Run any aim command to auto-download platform-tools, or set ADB_PATH."
            .into(),
    ))
}

fn find_adb_on_path() -> Option<PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    let name = if cfg!(target_os = "windows") {
        "adb.exe"
    } else {
        "adb"
    };

    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn aim_home() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| AimError::Toolchain("Could not determine home directory".into()))?;
    Ok(home.join(".aim"))
}

fn managed_adb_path() -> Result<PathBuf> {
    // Google's zip extracts to a `platform-tools/` subdirectory
    let path = aim_home()?.join("platform-tools").join("adb");
    Ok(path)
}

fn platform_tools_url() -> Result<&'static str> {
    if cfg!(target_os = "linux") {
        Ok(PLATFORM_TOOLS_URL_LINUX)
    } else if cfg!(target_os = "macos") {
        Ok(PLATFORM_TOOLS_URL_MACOS)
    } else {
        Err(AimError::Toolchain(
            "Auto-download not supported on this platform. Please install adb manually and set ADB_PATH.".into(),
        ))
    }
}

async fn download_platform_tools() -> Result<()> {
    let url = platform_tools_url()?;
    let dest_dir = aim_home()?;
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| AimError::Toolchain(format!("Failed to create ~/.aim: {}", e)))?;

    let zip_path = dest_dir.join("platform-tools-download.zip");

    eprintln!("adb not found. Downloading Android platform-tools...");

    // Download with progress
    download_file(url, &zip_path).await?;

    // Extract
    eprintln!("Extracting platform-tools...");
    extract_zip(&zip_path, &dest_dir)?;

    // Clean up zip
    let _ = std::fs::remove_file(&zip_path);

    // Set executable permission on Unix
    #[cfg(unix)]
    {
        let adb_bin = dest_dir.join("platform-tools").join("adb");
        if adb_bin.exists() {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&adb_bin, std::fs::Permissions::from_mode(0o755));
        }
    }

    eprintln!("platform-tools installed to ~/.aim/platform-tools/");
    Ok(())
}

async fn download_file(url: &str, dest: &Path) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await.map_err(|e| {
        AimError::Toolchain(format!(
            "Failed to download platform-tools: {}. Install adb manually or set ADB_PATH.",
            e
        ))
    })?;

    if !response.status().is_success() {
        return Err(AimError::Toolchain(format!(
            "Download failed with status: {}",
            response.status()
        )));
    }

    let total_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("=> "),
    );
    pb.set_message("Downloading");

    let mut file = std::fs::File::create(dest)
        .map_err(|e| AimError::Toolchain(format!("Failed to create temp file: {}", e)))?;

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk
            .map_err(|e| AimError::Toolchain(format!("Download interrupted: {}", e)))?;
        std::io::Write::write_all(&mut file, &chunk)
            .map_err(|e| AimError::Toolchain(format!("Failed to write to disk: {}", e)))?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("Downloaded");
    Ok(())
}

fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(zip_path)
        .map_err(|e| AimError::Toolchain(format!("Failed to open zip: {}", e)))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AimError::Toolchain(format!("Invalid zip file: {}", e)))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AimError::Toolchain(format!("Failed to read zip entry: {}", e)))?;

        let Some(path) = entry.enclosed_name() else {
            continue;
        };
        let out_path = dest_dir.join(path);

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| {
                AimError::Toolchain(format!("Failed to create directory: {}", e))
            })?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AimError::Toolchain(format!("Failed to create directory: {}", e))
                })?;
            }
            let mut outfile = std::fs::File::create(&out_path).map_err(|e| {
                AimError::Toolchain(format!("Failed to create file: {}", e))
            })?;
            std::io::copy(&mut entry, &mut outfile).map_err(|e| {
                AimError::Toolchain(format!("Failed to extract file: {}", e))
            })?;

            // Preserve Unix permissions from zip
            #[cfg(unix)]
            {
                if let Some(mode) = entry.unix_mode() {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(
                        &out_path,
                        std::fs::Permissions::from_mode(mode),
                    );
                }
            }
        }
    }

    Ok(())
}
