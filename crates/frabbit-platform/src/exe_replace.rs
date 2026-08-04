//! Replace the currently-running FRABBIT executable with a freshly-downloaded
//! one, in place, so the single-exe app can self-update.
//!
//! On Windows a running `.exe` can be *renamed* but not deleted or overwritten.
//! The `self-replace` crate performs the correct sequence — move the current
//! exe to a sidecar, put the new file at the original path, schedule the old
//! sidecar for deletion — so we wrap it rather than hand-rolling the dance.

use std::path::Path;

/// Returns `true` when the directory containing the running executable is
/// writable, i.e. an in-place self-replace can be attempted. FRABBIT is a
/// portable single exe, so the common case (Downloads / a portable folder) is
/// writable; a copy dropped into `C:\Program Files\…` is not, and the caller
/// falls back to opening the download page instead.
pub fn current_exe_dir_is_writable() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let Some(dir) = exe.parent() else {
        return false;
    };
    let probe = dir.join(".frabbit-write-test");
    match std::fs::File::create(&probe) {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// Replace the running executable with the file at `new_exe` (already
/// downloaded AND checksum-verified by the caller). On success the file at the
/// current exe path now holds the new binary; the old one is kept as a sidecar
/// until the OS can remove it. The process is untouched and keeps running on
/// the old image in memory — the caller relaunches to pick up the new version.
pub fn replace_running_exe(new_exe: &Path) -> std::io::Result<()> {
    self_replace::self_replace(new_exe)
}

#[cfg(test)]
mod tests {
    use super::current_exe_dir_is_writable;

    #[test]
    fn current_exe_dir_is_writable_returns_true_in_the_test_harness() {
        // The test binary lives under target/, which is writable; this mainly
        // guards against the probe panicking or mis-reporting on a normal dir.
        assert!(current_exe_dir_is_writable());
    }
}
