//! Replace the running FRABBIT `.app` bundle with a freshly-downloaded one, so
//! the macOS build can self-update the way the Windows build replaces its exe.
//!
//! macOS ships FRABBIT as an application bundle rather than a bare executable,
//! and the bundle carries more than the binary: `Contents/Info.plist` holds the
//! version string the Finder and Launch Services report, `Contents/Resources`
//! holds the `.lproj` layout VoiceOver reads to pick a voice, and the whole
//! directory is sealed by an ad-hoc code signature applied at build time.
//! Overwriting only `Contents/MacOS/frabbit` would leave every one of those
//! stale and break that seal, so a self-update swaps the entire bundle for the
//! one the release pipeline signed.
//!
//! Swapping while FRABBIT runs is safe: on Unix a running process holds its
//! image through a rename or a delete of the file it was launched from. The old
//! bundle can therefore be moved aside and deleted immediately — there is no
//! Windows-style leftover to sweep up on the next start.

use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
use std::process::Command;

#[derive(Debug)]
pub enum AppBundleError {
    /// The path handed in isn't a `.app` directory — the caller resolved the
    /// running bundle wrongly, or FRABBIT is running as a bare binary.
    NotABundle(PathBuf),
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// `ditto` refused to expand the downloaded archive. Carries its stderr:
    /// a truncated download that still matched its checksum is impossible, so
    /// this is nearly always a disk-space or permission problem worth showing.
    DittoFailed {
        archive: PathBuf,
        code: Option<i32>,
        stderr: String,
    },
    /// The archive expanded but held no `.app` — the release layout changed
    /// under us. Better to stop than to install something unrecognised.
    BundleNotFoundInArchive { archive: PathBuf },
    /// The extracted bundle is missing the parts every macOS app needs. Guards
    /// against swapping a working install for a broken one.
    IncompleteBundle { bundle: PathBuf },
    /// Bundle replacement is macOS-only.
    Unsupported,
}

impl std::fmt::Display for AppBundleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotABundle(path) => {
                write!(formatter, "{} is not a .app bundle", path.display())
            }
            Self::Io { path, source } => {
                write!(formatter, "{}: {source}", path.display())
            }
            Self::DittoFailed {
                archive,
                code,
                stderr,
            } => {
                let code = code.map_or_else(|| "unknown".to_string(), |code| code.to_string());
                write!(
                    formatter,
                    "could not expand {} (ditto exit {code}): {}",
                    archive.display(),
                    stderr.trim()
                )
            }
            Self::BundleNotFoundInArchive { archive } => {
                write!(formatter, "no .app bundle inside {}", archive.display())
            }
            Self::IncompleteBundle { bundle } => write!(
                formatter,
                "{} is missing Contents/Info.plist or Contents/MacOS",
                bundle.display()
            ),
            Self::Unsupported => {
                write!(
                    formatter,
                    "app-bundle replacement is only supported on macOS"
                )
            }
        }
    }
}

impl std::error::Error for AppBundleError {}

/// The `.app` bundle the running executable lives inside, or `None` when
/// FRABBIT was started as a bare binary (the CLI, `cargo run`, a download
/// dropped straight into a folder). Callers use this to decide between a
/// bundle swap and the plain executable replacement.
pub fn current_app_bundle() -> Option<PathBuf> {
    app_bundle_for_executable(&std::env::current_exe().ok()?)
}

/// Walk `…/Some.app/Contents/MacOS/<exe>` back up to `…/Some.app`. Split out
/// from [`current_app_bundle`] so the path logic is testable on every platform
/// without a real bundle on disk.
pub fn app_bundle_for_executable(executable: &Path) -> Option<PathBuf> {
    let macos_dir = executable.parent()?;
    if macos_dir.file_name()? != "MacOS" {
        return None;
    }
    let contents_dir = macos_dir.parent()?;
    if contents_dir.file_name()? != "Contents" {
        return None;
    }
    let bundle = contents_dir.parent()?;
    let is_app = bundle
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("app"));
    is_app.then(|| bundle.to_path_buf())
}

/// Returns `true` when the *directory holding* the bundle is writable, i.e. the
/// swap can be attempted. The bundle's own contents being writable is not
/// enough and not required: replacement renames directories inside the parent.
/// A copy under `/Applications` is writable for an admin user; one under a
/// read-only volume or another user's home is not, and the caller falls back to
/// opening the download page.
pub fn app_bundle_is_replaceable(bundle: &Path) -> bool {
    let Some(parent) = bundle.parent() else {
        return false;
    };
    let probe = parent.join(".frabbit-write-test");
    match std::fs::File::create(&probe) {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// Replace `bundle` with the `.app` inside `archive` (a `.app.zip` already
/// downloaded AND checksum-verified by the caller).
///
/// The archive is expanded with `ditto`, which — unlike `unzip` — preserves the
/// extended attributes and resource forks that carry a bundle's code signature,
/// so the installed copy stays exactly as signed by the release pipeline.
///
/// The swap itself is two renames inside the bundle's parent directory: the old
/// bundle moves to a hidden sibling, the new one takes its place. If the second
/// rename fails the first is undone, so the only reachable outcomes are "fully
/// updated" and "untouched". On success the old bundle and the staging
/// directory are deleted immediately.
#[cfg(target_os = "macos")]
pub fn replace_app_bundle(archive: &Path, bundle: &Path) -> Result<(), AppBundleError> {
    let parent = bundle
        .parent()
        .ok_or_else(|| AppBundleError::NotABundle(bundle.to_path_buf()))?;
    let bundle_name = bundle
        .file_name()
        .ok_or_else(|| AppBundleError::NotABundle(bundle.to_path_buf()))?;

    // Stage inside the bundle's parent, not a temp dir: the final move has to
    // be a rename, and a rename only works within one volume.
    let staging = parent.join(format!(".frabbit-update-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(|source| AppBundleError::Io {
        path: staging.clone(),
        source,
    })?;

    let result = stage_and_swap(archive, bundle, &staging, bundle_name);
    let _ = std::fs::remove_dir_all(&staging);
    result
}

#[cfg(target_os = "macos")]
fn stage_and_swap(
    archive: &Path,
    bundle: &Path,
    staging: &Path,
    bundle_name: &std::ffi::OsStr,
) -> Result<(), AppBundleError> {
    let output = Command::new("/usr/bin/ditto")
        .arg("-x")
        .arg("-k")
        .arg(archive)
        .arg(staging)
        .output()
        .map_err(|source| AppBundleError::Io {
            path: PathBuf::from("/usr/bin/ditto"),
            source,
        })?;
    if !output.status.success() {
        return Err(AppBundleError::DittoFailed {
            archive: archive.to_path_buf(),
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    // The release zip nests the bundle beside the "Open Me First.command"
    // helper, so search rather than assuming a top-level `Frabbit.app`.
    let new_bundle =
        crate::disk_image::find_app_bundle_in_directory(staging, &bundle_name.to_string_lossy())
            .ok_or_else(|| AppBundleError::BundleNotFoundInArchive {
                archive: archive.to_path_buf(),
            })?;
    if !new_bundle.join("Contents/Info.plist").is_file()
        || !new_bundle.join("Contents/MacOS").is_dir()
    {
        return Err(AppBundleError::IncompleteBundle { bundle: new_bundle });
    }

    // FRABBIT downloads the archive itself, so it carries no quarantine flag —
    // but clear it anyway, cheaply, in case the file arrived some other way
    // (a user pointing FRABBIT at a manual download). Best-effort: a bundle
    // that never had the attribute makes xattr exit non-zero on some releases.
    let _ = Command::new("/usr/bin/xattr")
        .arg("-dr")
        .arg("com.apple.quarantine")
        .arg(&new_bundle)
        .output();

    let previous = staging.join("previous.app");
    std::fs::rename(bundle, &previous).map_err(|source| AppBundleError::Io {
        path: bundle.to_path_buf(),
        source,
    })?;
    if let Err(source) = std::fs::rename(&new_bundle, bundle) {
        // Put the working install back before reporting the failure.
        let _ = std::fs::rename(&previous, bundle);
        return Err(AppBundleError::Io {
            path: bundle.to_path_buf(),
            source,
        });
    }
    // The running process keeps its image after this, so the old bundle can go
    // now instead of lingering until the next start.
    let _ = std::fs::remove_dir_all(&previous);
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn replace_app_bundle(_archive: &Path, _bundle: &Path) -> Result<(), AppBundleError> {
    Err(AppBundleError::Unsupported)
}

/// Relaunch `bundle` through Launch Services and return whether the launch was
/// accepted. `-n` forces a fresh instance because the outgoing one is still
/// running, and Launch Services puts the new window in the foreground so
/// VoiceOver follows it there — which a direct spawn of the inner binary does
/// not do. `--env` is how the two variables survive the handoff: the launched
/// app inherits launchd's environment, not ours.
#[cfg(target_os = "macos")]
pub fn relaunch_app_bundle(bundle: &Path) -> bool {
    let mut command = Command::new("/usr/bin/open");
    command.arg("-n").arg("--env").arg("FRABBIT_RELAUNCHED=1");
    if let Ok(locale) = std::env::var("FRABBIT_LOCALE") {
        command.arg("--env").arg(format!("FRABBIT_LOCALE={locale}"));
    }
    command.arg(bundle);
    command.status().is_ok_and(|status| status.success())
}

#[cfg(not(target_os = "macos"))]
pub fn relaunch_app_bundle(_bundle: &Path) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_the_bundle_from_an_executable_inside_it() {
        let bundle = app_bundle_for_executable(Path::new(
            "/Applications/Frabbit.app/Contents/MacOS/frabbit",
        ));
        assert_eq!(bundle, Some(PathBuf::from("/Applications/Frabbit.app")));
    }

    #[test]
    fn a_bare_executable_has_no_bundle() {
        assert_eq!(
            app_bundle_for_executable(Path::new("/Users/someone/Downloads/frabbit")),
            None
        );
        // A binary that merely sits in a directory called MacOS isn't in a
        // bundle either — the .app suffix is what makes it one.
        assert_eq!(
            app_bundle_for_executable(Path::new("/tmp/Frabbit/Contents/MacOS/frabbit")),
            None
        );
    }

    #[test]
    fn the_app_suffix_check_ignores_case() {
        assert_eq!(
            app_bundle_for_executable(Path::new("/tmp/Frabbit.APP/Contents/MacOS/frabbit")),
            Some(PathBuf::from("/tmp/Frabbit.APP"))
        );
    }

    /// Build a minimal bundle whose binary prints `marker`, mirroring the
    /// layout `packaging/macos/build-bundle.sh` produces.
    #[cfg(target_os = "macos")]
    fn write_bundle(path: &Path, marker: &str) {
        std::fs::create_dir_all(path.join("Contents/MacOS")).unwrap();
        std::fs::write(
            path.join("Contents/Info.plist"),
            format!("<plist><dict><key>Version</key><string>{marker}</string></dict></plist>"),
        )
        .unwrap();
        std::fs::write(path.join("Contents/MacOS/frabbit"), marker).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn replaces_a_bundle_from_a_zip_that_nests_it_beside_a_helper() {
        let root = tempfile::tempdir().unwrap();
        let installed = root.path().join("Frabbit.app");
        write_bundle(&installed, "old");

        // Mirror the release layout: Frabbit.app inside a wrapper folder,
        // next to the unquarantine helper, zipped with ditto.
        let source = root.path().join("source/Frabbit");
        std::fs::create_dir_all(&source).unwrap();
        write_bundle(&source.join("Frabbit.app"), "new");
        std::fs::write(source.join("Open Me First.command"), "#!/bin/bash\n").unwrap();
        let archive = root.path().join("update.app.zip");
        let status = Command::new("/usr/bin/ditto")
            .args(["-c", "-k", "--sequesterRsrc", "--keepParent"])
            .arg(&source)
            .arg(&archive)
            .status()
            .unwrap();
        assert!(status.success());

        replace_app_bundle(&archive, &installed).unwrap();

        assert_eq!(
            std::fs::read_to_string(installed.join("Contents/MacOS/frabbit")).unwrap(),
            "new"
        );
        assert!(
            std::fs::read_to_string(installed.join("Contents/Info.plist"))
                .unwrap()
                .contains("new"),
            "the whole bundle should be replaced, Info.plist included"
        );
        // Nothing left behind next to the bundle.
        let leftovers: Vec<_> = std::fs::read_dir(root.path())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with(".frabbit-update-"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "staging dirs left behind: {leftovers:?}"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn a_zip_without_a_bundle_leaves_the_install_untouched() {
        let root = tempfile::tempdir().unwrap();
        let installed = root.path().join("Frabbit.app");
        write_bundle(&installed, "old");

        let source = root.path().join("source");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("readme.txt"), "no bundle here").unwrap();
        let archive = root.path().join("update.app.zip");
        Command::new("/usr/bin/ditto")
            .args(["-c", "-k", "--keepParent"])
            .arg(&source)
            .arg(&archive)
            .status()
            .unwrap();

        let error = replace_app_bundle(&archive, &installed).unwrap_err();
        assert!(matches!(
            error,
            AppBundleError::BundleNotFoundInArchive { .. }
        ));
        assert_eq!(
            std::fs::read_to_string(installed.join("Contents/MacOS/frabbit")).unwrap(),
            "old",
            "a failed update must leave the working install in place"
        );
    }
}
