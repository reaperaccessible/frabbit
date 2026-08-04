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

/// Delete the leftover files a self-replace leaves behind in the executable's
/// directory: the renamed old binary (`*.__relocated__.exe`), which Windows
/// otherwise only removes on reboot, plus any stray `*.__temp__.exe` /
/// `*.__selfdelete__.exe` helpers. Safe to call at startup: by then the old
/// process has exited, so the old binary is no longer locked and deletes
/// immediately. Best-effort — failures are ignored.
pub fn cleanup_update_leftovers() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(dir) = exe.parent() else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.ends_with(".__relocated__.exe")
            || name.ends_with(".__temp__.exe")
            || name.ends_with(".__selfdelete__.exe")
        {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// Grant the process we're about to spawn the right to take the foreground
/// window, so the relaunched FRABBIT becomes the active window and the screen
/// reader follows it. Must be called from the current (foreground) process
/// just before spawning the replacement. No-op off Windows.
#[cfg(target_os = "windows")]
pub fn allow_foreground_for_relaunch() {
    // ASFW_ANY (-1): allow any process to set the foreground window.
    const ASFW_ANY: u32 = 0xFFFF_FFFF;
    unsafe {
        let _ = windows_sys::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow(ASFW_ANY);
    }
}
#[cfg(not(target_os = "windows"))]
pub fn allow_foreground_for_relaunch() {}

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
