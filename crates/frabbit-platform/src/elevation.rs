//! Elevated process launcher.
//!
//! On Windows, some vendor installers — notably the JAWS-for-REAPER scripts
//! NSIS package — declare `RequestExecutionLevel admin` in their script
//! because they need to write into `C:\Program Files (x86)\…`. A normal
//! `CreateProcess` call from an unelevated parent never triggers UAC, so on
//! default Windows the installer silently no-ops in `/S` silent mode. We
//! work around that by launching the installer through `ShellExecuteExW`
//! with the `runas` verb, which always raises the UAC consent dialog when
//! the user is not already elevated, then waiting on the returned process
//! handle for the exit code.
//!
//! On macOS we wrap the same call through `osascript`'s `do shell script
//! "..." with administrator privileges` so the system raises its native
//! AuthorizationServices dialog. That's the screen-reader-friendly path
//! used by Apple's own tooling — sudo can't read passwords from a GUI
//! parent and `SMJobBless` would require a separate signed helper bundle
//! FRABBIT doesn't have. Today only Surge XT's `installer -pkg` flow uses
//! this path.
//!
//! Other targets compile to a stub that returns an `Unsupported` error.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ElevationError {
    /// `ShellExecuteExW` failed before producing a process handle. `code` is
    /// the OS-reported `GetLastError()` value.
    LaunchFailed { program: PathBuf, code: u32 },
    /// `WaitForSingleObject` or `GetExitCodeProcess` failed, or the process
    /// terminated abnormally so we have no exit code to report.
    WaitFailed { program: PathBuf, message: String },
    /// The user dismissed the elevation prompt — the UAC consent dialog on
    /// Windows, the admin authorization dialog on macOS. Distinct from a
    /// generic launch failure so the caller can surface a clearer message.
    UserCancelledElevation { program: PathBuf },
    /// Compiled on a target that has no elevation primitive (Windows and
    /// macOS are the two that do).
    Unsupported,
}

impl std::fmt::Display for ElevationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LaunchFailed { program, code } => write!(
                f,
                "ShellExecuteExW(runas) failed for {} (Win32 error {code})",
                program.display()
            ),
            Self::WaitFailed { program, message } => write!(
                f,
                "could not read exit status for elevated process {}: {message}",
                program.display()
            ),
            // Raised on both Windows (UAC declined) and macOS (osascript
            // reporting "User canceled"), so the wording stays neutral.
            Self::UserCancelledElevation { program } => write!(
                f,
                "the administrator approval prompt for {} was cancelled or declined",
                program.display()
            ),
            Self::Unsupported => write!(
                f,
                "elevated process launch is not supported on this platform"
            ),
        }
    }
}

impl std::error::Error for ElevationError {}

/// Whether a `requires_elevation` installer must be launched through the
/// `runas` verb (raising a UAC prompt) rather than a plain, direct
/// `CreateProcess`.
///
/// The decision keys off the process token's **elevation TYPE**, not the
/// unreliable `TokenIsElevated` flag:
///
/// * `Full` — already elevated (Run as administrator, or elevated under
///   active UAC): launch directly, no prompt.
/// * `Limited` — a *filtered* admin token under active UAC: `runas` is
///   needed to obtain the full token via the consent prompt.
/// * `Default` — no split token exists. This covers BOTH a UAC-disabled
///   admin (whose default token is already full → launch directly) and an
///   ordinary standard user (who genuinely needs `runas`). We disambiguate
///   with an Administrators-group membership check.
///
/// This fixes the case that mislabelled a working install as cancelled:
/// with UAC disabled (`EnableLUA=0`), an administrator's token is `Default`
/// with `TokenIsElevated == 0`. The old check saw "not elevated" and forced
/// `runas`, but there is no elevation broker when UAC is off, so
/// `ShellExecuteEx(runas)` returned `ERROR_CANCELLED` (1223) and the
/// installer never ran — even though a direct launch would have installed
/// fine with the already-full admin token. Always `false` off Windows.
pub fn needs_runas_to_elevate() -> bool {
    platform_needs_runas_to_elevate()
}

#[cfg(windows)]
fn platform_needs_runas_to_elevate() -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TOKEN_QUERY, TokenElevationType, TokenElevationTypeFull,
        TokenElevationTypeLimited,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let elevation_type = unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            // Can't tell — fail toward a direct launch rather than a
            // possibly-doomed runas.
            return false;
        }
        let mut ty: i32 = 0;
        let mut return_length = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevationType,
            &mut ty as *mut i32 as *mut core::ffi::c_void,
            std::mem::size_of::<i32>() as u32,
            &mut return_length,
        );
        CloseHandle(token);
        if ok == 0 {
            return false;
        }
        ty
    };

    if elevation_type == TokenElevationTypeFull as i32 {
        false
    } else if elevation_type == TokenElevationTypeLimited as i32 {
        true
    } else {
        // Default: UAC-off admin → already full token → direct;
        // standard user → needs runas.
        !current_token_is_admin()
    }
}

/// `true` if the current process's effective token has the Administrators
/// group active (not present-but-deny-only). A UAC-off admin reads as a
/// member; a filtered admin token reads as a non-member (Administrators is
/// deny-only in that token); a standard user reads as a non-member.
#[cfg(windows)]
fn current_token_is_admin() -> bool {
    use windows_sys::Win32::Foundation::FALSE;
    use windows_sys::Win32::Security::{
        AllocateAndInitializeSid, CheckTokenMembership, FreeSid, PSID, SECURITY_NT_AUTHORITY,
    };
    use windows_sys::Win32::System::SystemServices::{
        DOMAIN_ALIAS_RID_ADMINS, SECURITY_BUILTIN_DOMAIN_RID,
    };

    unsafe {
        let mut authority = SECURITY_NT_AUTHORITY;
        let mut admins: PSID = std::ptr::null_mut();
        if AllocateAndInitializeSid(
            &mut authority,
            2,
            SECURITY_BUILTIN_DOMAIN_RID as u32,
            DOMAIN_ALIAS_RID_ADMINS as u32,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut admins,
        ) == 0
        {
            return false;
        }
        let mut is_member = FALSE;
        // NULL token = the process's effective token.
        let ok = CheckTokenMembership(std::ptr::null_mut(), admins, &mut is_member);
        FreeSid(admins);
        ok != 0 && is_member != FALSE
    }
}

#[cfg(not(windows))]
fn platform_needs_runas_to_elevate() -> bool {
    false
}

/// Launch `program` with `arguments` under UAC elevation and block until it
/// exits. Returns the process exit code (`Some(n)`) on a clean exit, or
/// `None` if the OS could not return one (rare). Working directory may be
/// `None` to inherit the current directory.
#[cfg_attr(not(windows), allow(unused_variables))]
pub fn run_elevated_and_wait(
    program: &Path,
    arguments: &[String],
    working_directory: Option<&Path>,
) -> Result<Option<i32>, ElevationError> {
    platform_run_elevated_and_wait(program, arguments, working_directory)
}

#[cfg(windows)]
fn platform_run_elevated_and_wait(
    program: &Path,
    arguments: &[String],
    working_directory: Option<&Path>,
) -> Result<Option<i32>, ElevationError> {
    use std::os::windows::ffi::OsStrExt;

    use windows_sys::Win32::Foundation::{CloseHandle, ERROR_CANCELLED, GetLastError, WAIT_FAILED};
    use windows_sys::Win32::System::Com::{
        COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize,
    };
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, INFINITE, WaitForSingleObject,
    };
    use windows_sys::Win32::UI::Shell::{
        SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        ASFW_ANY, AllowSetForegroundWindow, SW_SHOWNORMAL,
    };

    // The install pipeline runs off the UI thread. `ShellExecuteEx`'s `runas`
    // verb can delegate to shell/COM code, and MSDN requires COM to be
    // initialized on the calling thread; without it the UAC consent UI can
    // fail to surface and the call comes back as ERROR_CANCELLED even though
    // the user never saw a prompt. Initialize an apartment for the duration
    // of this call and balance it with `CoUninitialize` on every exit path.
    struct ComGuard {
        should_uninitialize: bool,
    }
    impl Drop for ComGuard {
        fn drop(&mut self) {
            if self.should_uninitialize {
                unsafe { CoUninitialize() };
            }
        }
    }
    let com_hr = unsafe { CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32) };
    // SUCCEEDED(hr) (S_OK / S_FALSE, both >= 0) means our call added a
    // reference we own and must release. A negative HRESULT (e.g.
    // RPC_E_CHANGED_MODE) means another apartment already owns the thread —
    // don't uninitialize what we didn't initialize.
    let _com_guard = ComGuard {
        should_uninitialize: com_hr >= 0,
    };

    // Let the elevation/consent UI take the foreground so a screen reader
    // reliably announces and focuses it, instead of it flashing as a taskbar
    // button the (blind) user can't see or act on.
    unsafe { AllowSetForegroundWindow(ASFW_ANY) };

    let verb_w: Vec<u16> = OsStr::new("runas").encode_wide().chain(Some(0)).collect();
    let program_w: Vec<u16> = program.as_os_str().encode_wide().chain(Some(0)).collect();
    let parameters = quote_arguments(arguments);
    let parameters_w: Vec<u16> = OsStr::new(&parameters)
        .encode_wide()
        .chain(Some(0))
        .collect();
    let working_directory_w: Option<Vec<u16>> =
        working_directory.map(|path| path.as_os_str().encode_wide().chain(Some(0)).collect());

    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        // NOCLOSEPROCESS: keep the process handle so we can wait on it.
        // NOASYNC: block until the (possibly COM/DDE-backed) operation is
        // fully done — required when calling from a thread without a message
        // loop, otherwise ShellExecuteEx can return before the elevated
        // process is actually launched.
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
        hwnd: std::ptr::null_mut(),
        lpVerb: verb_w.as_ptr(),
        lpFile: program_w.as_ptr(),
        lpParameters: if parameters.is_empty() {
            std::ptr::null()
        } else {
            parameters_w.as_ptr()
        },
        lpDirectory: working_directory_w
            .as_ref()
            .map(|w| w.as_ptr())
            .unwrap_or(std::ptr::null()),
        nShow: SW_SHOWNORMAL,
        hInstApp: std::ptr::null_mut(),
        lpIDList: std::ptr::null_mut(),
        lpClass: std::ptr::null(),
        hkeyClass: std::ptr::null_mut(),
        dwHotKey: 0,
        Anonymous: unsafe { std::mem::zeroed() },
        hProcess: std::ptr::null_mut(),
    };

    let ok = unsafe { ShellExecuteExW(&mut info) };
    if ok == 0 {
        let code = unsafe { GetLastError() };
        // Windows returns ERROR_CANCELLED (1223) when the user dismisses the
        // UAC consent prompt; surface that as a distinct error so the wizard
        // can prompt them to re-run and approve.
        if code == ERROR_CANCELLED {
            return Err(ElevationError::UserCancelledElevation {
                program: program.to_path_buf(),
            });
        }
        return Err(ElevationError::LaunchFailed {
            program: program.to_path_buf(),
            code,
        });
    }

    if info.hProcess.is_null() {
        // Some shell verbs return success without a process handle (e.g.
        // when the file extension was handled by the shell instead of by
        // CreateProcess). For our use case — launching a real `.exe` — that
        // would be a misconfiguration; treat it as a wait failure so the
        // caller doesn't silently report success.
        return Err(ElevationError::WaitFailed {
            program: program.to_path_buf(),
            message: "ShellExecuteExW returned no process handle".to_string(),
        });
    }

    let wait_status = unsafe { WaitForSingleObject(info.hProcess, INFINITE) };
    if wait_status == WAIT_FAILED {
        let code = unsafe { GetLastError() };
        unsafe {
            CloseHandle(info.hProcess);
        }
        return Err(ElevationError::WaitFailed {
            program: program.to_path_buf(),
            message: format!("WaitForSingleObject failed (Win32 error {code})"),
        });
    }

    let mut exit_code: u32 = 0;
    let ok = unsafe { GetExitCodeProcess(info.hProcess, &mut exit_code) };
    unsafe {
        CloseHandle(info.hProcess);
    }
    if ok == 0 {
        let code = unsafe { GetLastError() };
        return Err(ElevationError::WaitFailed {
            program: program.to_path_buf(),
            message: format!("GetExitCodeProcess failed (Win32 error {code})"),
        });
    }

    Ok(Some(exit_code as i32))
}

#[cfg(target_os = "macos")]
fn platform_run_elevated_and_wait(
    program: &Path,
    arguments: &[String],
    working_directory: Option<&Path>,
) -> Result<Option<i32>, ElevationError> {
    use std::process::Command;

    // Re-emit the command as an AppleScript `do shell script` literal.
    // `osascript` is the standard way to raise macOS' native admin prompt
    // without bundling a signed helper. The script string must escape any
    // embedded double-quotes; everything else (spaces, colons, parens) is
    // safe inside a quoted AppleScript string.
    let command_line = if arguments.is_empty() {
        applescript_quote(&program.display().to_string())
    } else {
        let joined = std::iter::once(program.display().to_string())
            .chain(arguments.iter().cloned())
            .map(|argument| applescript_quote(&argument))
            .collect::<Vec<_>>()
            .join(" & space & ");
        joined
    };
    let script = format!(
        "do shell script ({command_line}) with administrator privileges",
        command_line = command_line
    );

    let mut command = Command::new("/usr/bin/osascript");
    command.arg("-e").arg(&script);
    if let Some(working_directory) = working_directory {
        command.current_dir(working_directory);
    }

    let output = command
        .output()
        .map_err(|err| ElevationError::LaunchFailed {
            program: program.to_path_buf(),
            code: err.raw_os_error().unwrap_or(0) as u32,
        })?;

    if output.status.success() {
        return Ok(Some(0));
    }

    // osascript exit 1 + stderr containing "User cancelled" → the user
    // dismissed the system admin prompt. Surface as a distinct error so
    // the wizard can prompt them to re-run and approve, matching the
    // Windows ERROR_CANCELLED branch.
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("User canceled") || stderr.contains("User cancelled") {
        return Err(ElevationError::UserCancelledElevation {
            program: program.to_path_buf(),
        });
    }

    Ok(output.status.code())
}

#[cfg(not(any(windows, target_os = "macos")))]
fn platform_run_elevated_and_wait(
    _program: &Path,
    _arguments: &[String],
    _working_directory: Option<&Path>,
) -> Result<Option<i32>, ElevationError> {
    Err(ElevationError::Unsupported)
}

/// Escape a literal command-line token for embedding inside an
/// AppleScript double-quoted string. Internal `"` and `\` are escaped so
/// the resulting string round-trips through `do shell script` exactly.
#[cfg(target_os = "macos")]
fn applescript_quote(argument: &str) -> String {
    let escaped = argument.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Quote each argument the way `ShellExecuteEx` expects (one space-joined
/// command-line string), wrapping arguments containing whitespace or quotes
/// in double-quotes and escaping internal quotes.
fn quote_arguments(arguments: &[String]) -> String {
    arguments
        .iter()
        .map(|argument| quote_one(argument))
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote_one(argument: &str) -> String {
    if !argument.is_empty()
        && !argument.contains(|ch: char| ch.is_whitespace() || ch == '"' || ch == '\\')
    {
        return argument.to_string();
    }
    let escaped = argument.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::quote_arguments;

    #[test]
    fn quotes_arguments_with_whitespace() {
        let line = quote_arguments(&[
            "/S".to_string(),
            "/D=C:\\Program Files (x86)\\Foo".to_string(),
        ]);
        assert_eq!(line, "/S \"/D=C:\\\\Program Files (x86)\\\\Foo\"");
    }

    #[test]
    fn skips_quoting_for_simple_arguments() {
        assert_eq!(quote_arguments(&["/S".to_string()]), "/S");
        assert_eq!(quote_arguments(&[]), "");
    }
}
