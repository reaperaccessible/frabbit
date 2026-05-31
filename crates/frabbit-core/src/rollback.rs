use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{IoPathContext, JsonPathContext, Result};

pub const BACKUP_MANIFEST_FILE: &str = "backup-manifest.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupManifest {
    pub schema_version: u32,
    pub frabbit_version: String,
    pub created_at: String,
    pub reason: String,
    pub files: Vec<BackupManifestFile>,
    pub receipt_backup_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupManifestFile {
    pub package_id: Option<String>,
    pub original_path: PathBuf,
    pub backup_path: PathBuf,
    pub size: u64,
    pub sha256: String,
}

pub fn save_backup_manifest(backup_set: &Path, manifest: &BackupManifest) -> Result<PathBuf> {
    fs::create_dir_all(backup_set).with_path(backup_set)?;
    let path = backup_set.join(BACKUP_MANIFEST_FILE);
    let content = serde_json::to_string_pretty(manifest).with_json_path(&path)?;
    fs::write(&path, content).with_path(&path)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{BackupManifest, BackupManifestFile, save_backup_manifest};

    #[test]
    fn save_backup_manifest_creates_file() {
        let dir = tempdir().unwrap();
        let backup_set = dir.path().join("FRABBIT/backups/unix-1");

        let path = save_backup_manifest(
            &backup_set,
            &BackupManifest {
                schema_version: 1,
                frabbit_version: "0.1.0".to_string(),
                created_at: "unix-1".to_string(),
                reason: "install-replacement".to_string(),
                files: vec![BackupManifestFile {
                    package_id: Some("reapack".to_string()),
                    original_path: std::path::PathBuf::from("UserPlugins/reaper_reapack-x64.dll"),
                    backup_path: backup_set.join("UserPlugins/reaper_reapack-x64.dll"),
                    size: 3,
                    sha256: "abc123".to_string(),
                }],
                receipt_backup_path: None,
            },
        )
        .unwrap();

        assert!(path.is_file());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("reapack"));
    }
}
