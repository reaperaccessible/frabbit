use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::{FrabbitError, IoPathContext, Result};
use crate::progress::{ProgressEvent, ProgressReporter};

const CSI_RELEASE_URL: &str =
    "https://github.com/reaperaccessible/CSI/releases/latest/download/CSI.zip";

const CSI_REAPACK_REPO_URL: &str = "https://github.com/reaperaccessible/CSI/raw/main/index.xml";

const CSI_REAPACK_REPO_NAME: &str = "ReaperAccessible CSI";

const CSI_FOLDER_NAME: &str = "CSI For Behringer X-Touch Universal";

const VERSION_FILE_NAME: &str = ".frabbit-version";

pub fn csi_reapack_repo_url() -> &'static str {
    CSI_REAPACK_REPO_URL
}

pub fn csi_reapack_repo_name() -> &'static str {
    CSI_REAPACK_REPO_NAME
}

pub fn install_csi(resource_path: &Path, progress: &ProgressReporter) -> Result<()> {
    let documents_dir = documents_folder()?;
    let csi_dest = documents_dir.join(CSI_FOLDER_NAME);

    progress.report(ProgressEvent::CsiDownloadStarted);
    let zip_bytes = download_csi_zip()?;
    progress.report(ProgressEvent::CsiDownloadCompleted);

    let reader = std::io::Cursor::new(&zip_bytes);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| FrabbitError::RemoteData {
        url: CSI_RELEASE_URL.to_string(),
        message: format!("failed to open CSI zip: {e}"),
    })?;

    extract_csi_archive(&mut archive, &csi_dest, resource_path)?;

    let version = env!("CARGO_PKG_VERSION");
    let version_file = csi_dest.join(VERSION_FILE_NAME);
    fs::create_dir_all(&csi_dest).with_path(&csi_dest)?;
    fs::write(&version_file, version).with_path(&version_file)?;

    progress.report(ProgressEvent::CsiInstallCompleted);
    Ok(())
}

fn download_csi_zip() -> Result<Vec<u8>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!(
            "FRABBIT/",
            env!("CARGO_PKG_VERSION"),
            " (+https://github.com/ReaperAccessible/frabbit)"
        ))
        .build()
        .map_err(|e| FrabbitError::RemoteData {
            url: CSI_RELEASE_URL.to_string(),
            message: format!("failed to build HTTP client: {e}"),
        })?;

    let response = client
        .get(CSI_RELEASE_URL)
        .send()
        .map_err(|e| FrabbitError::RemoteData {
            url: CSI_RELEASE_URL.to_string(),
            message: format!("failed to download CSI: {e}"),
        })?;

    if !response.status().is_success() {
        return Err(FrabbitError::RemoteData {
            url: CSI_RELEASE_URL.to_string(),
            message: format!("HTTP {}", response.status()),
        });
    }

    response
        .bytes()
        .map(|b| b.to_vec())
        .map_err(|e| FrabbitError::RemoteData {
            url: CSI_RELEASE_URL.to_string(),
            message: format!("failed to read response: {e}"),
        })
}

fn extract_csi_archive(
    archive: &mut zip::ZipArchive<std::io::Cursor<&Vec<u8>>>,
    documents_dest: &Path,
    resource_path: &Path,
) -> Result<()> {
    let user_plugins_dir = resource_path.join("UserPlugins");
    fs::create_dir_all(&user_plugins_dir).with_path(&user_plugins_dir)?;

    let csi_resource_dir = resource_path.join("CSI");

    let prefix = format!("{}/", CSI_FOLDER_NAME);

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| FrabbitError::RemoteData {
            url: CSI_RELEASE_URL.to_string(),
            message: format!("failed to read zip entry {i}: {e}"),
        })?;

        let raw_name = file.name().to_string();

        let relative = if let Some(stripped) = raw_name.strip_prefix(&prefix) {
            stripped.to_string()
        } else {
            raw_name.clone()
        };

        if relative.is_empty() {
            continue;
        }

        // Route files to their correct destination
        if relative.starts_with("DLL file/") {
            if let Some(filename) = relative.strip_prefix("DLL file/") {
                if !filename.is_empty() && !file.is_dir() {
                    let dest = user_plugins_dir.join(filename);
                    write_zip_entry(&mut file, &dest)?;
                }
            }
        } else if relative.starts_with("CSI/") {
            let sub_path = relative.strip_prefix("CSI/").unwrap_or(&relative);
            if !sub_path.is_empty() {
                let dest = csi_resource_dir.join(sub_path);
                if file.is_dir() {
                    fs::create_dir_all(&dest).with_path(&dest)?;
                } else {
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent).with_path(parent)?;
                    }
                    write_zip_entry(&mut file, &dest)?;
                }
            }
        } else {
            // Everything else goes to Documents/CSI For Behringer X-Touch Universal/
            let dest = documents_dest.join(&relative);
            if file.is_dir() {
                fs::create_dir_all(&dest).with_path(&dest)?;
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).with_path(parent)?;
                }
                write_zip_entry(&mut file, &dest)?;
            }
        }
    }

    Ok(())
}

fn write_zip_entry(file: &mut zip::read::ZipFile, dest: &Path) -> Result<()> {
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).with_path(dest)?;
    fs::write(dest, &buf).with_path(dest)?;
    Ok(())
}

fn documents_folder() -> Result<PathBuf> {
    let userprofile = std::env::var("USERPROFILE").map_err(|_| FrabbitError::RemoteData {
        url: String::new(),
        message: "USERPROFILE environment variable not set".to_string(),
    })?;
    Ok(PathBuf::from(userprofile).join("Documents"))
}

pub fn installed_csi_version() -> Option<String> {
    let documents_dir = std::env::var("USERPROFILE").ok()?;
    let version_file = PathBuf::from(documents_dir)
        .join("Documents")
        .join(CSI_FOLDER_NAME)
        .join(VERSION_FILE_NAME);
    fs::read_to_string(&version_file)
        .ok()
        .map(|s| s.trim().to_string())
}
