use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::{FrabbitError, IoPathContext, Result};
use crate::package::embedded_package_manifest;

/// Run manifest-driven post-install steps for any package. Reads the embedded
/// manifest, finds the spec by `package_id`, and executes whatever
/// `post_install_*` fields are set. Returns `Ok(())` immediately when no
/// post-install fields are configured — this is the common case for most
/// packages, so calling this function unconditionally is cheap.
pub fn run_manifest_post_install(
    cached_zip_path: &Path,
    package_id: &str,
    resource_path: &Path,
    version: &str,
) -> Result<()> {
    let manifest = embedded_package_manifest();
    let Some(spec) = manifest.packages.iter().find(|p| p.id == package_id) else {
        return Ok(());
    };

    let has_zip_routes = !spec.post_install_zip_routes.is_empty();
    let has_version_file = spec.post_install_version_file.is_some();
    let has_reapack_repo = spec.post_install_reapack_repo.is_some();

    if !has_zip_routes && !has_version_file && !has_reapack_repo {
        return Ok(());
    }

    let documents_dir = documents_folder()?;

    if has_zip_routes {
        let zip_bytes = fs::read(cached_zip_path).with_path(cached_zip_path)?;
        let reader = std::io::Cursor::new(&zip_bytes);
        let mut archive = zip::ZipArchive::new(reader).map_err(|e| FrabbitError::RemoteData {
            url: cached_zip_path.display().to_string(),
            message: format!("failed to open zip: {e}"),
        })?;

        extract_zip_routes(
            &mut archive,
            &spec.post_install_zip_routes,
            resource_path,
            &documents_dir,
        )?;
    }

    if let Some(version_file_path) = &spec.post_install_version_file {
        let dest = documents_dir.join(version_file_path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).with_path(parent)?;
        }
        fs::write(&dest, version).with_path(&dest)?;
    }

    if let Some(repo) = &spec.post_install_reapack_repo {
        let _ = crate::reapack::upsert_remote(resource_path, &repo.name, &repo.url);
    }

    Ok(())
}

fn extract_zip_routes(
    archive: &mut zip::ZipArchive<std::io::Cursor<&Vec<u8>>>,
    routes: &[crate::package::ZipRoute],
    resource_path: &Path,
    documents_dir: &Path,
) -> Result<()> {
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| FrabbitError::RemoteData {
            url: String::new(),
            message: format!("failed to read zip entry {i}: {e}"),
        })?;

        let raw_name = file.name().to_string();

        // Check skip routes first (prefixed with `!`).
        let should_skip = routes.iter().any(|route| {
            if let Some(prefix) = route.zip_prefix.strip_prefix('!') {
                raw_name.starts_with(prefix)
            } else {
                false
            }
        });
        if should_skip {
            continue;
        }

        // Find the first matching non-skip route.
        let mut matched = false;
        for route in routes {
            if route.zip_prefix.starts_with('!') {
                continue;
            }
            if !raw_name.starts_with(&route.zip_prefix) {
                continue;
            }

            let relative = &raw_name[route.zip_prefix.len()..];
            if relative.is_empty() {
                matched = true;
                break;
            }

            let dest_base = resolve_destination(&route.destination, resource_path, documents_dir);
            let dest = dest_base.join(relative);

            if file.is_dir() {
                fs::create_dir_all(&dest).with_path(&dest)?;
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).with_path(parent)?;
                }
                write_zip_entry(&mut file, &dest)?;
            }
            matched = true;
            break;
        }

        // If no route matched, skip the entry.
        let _ = matched;
    }

    Ok(())
}

fn resolve_destination(destination: &str, resource_path: &Path, documents_dir: &Path) -> PathBuf {
    let resolved = destination
        .replace("{resource_path}", &resource_path.display().to_string())
        .replace("{documents}", &documents_dir.display().to_string());
    PathBuf::from(resolved)
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
