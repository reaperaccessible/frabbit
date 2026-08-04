use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use crate::Result;
use crate::archive::extract_osara_macos_assets;
use crate::disk_image::{install_app_bundle_from_disk_image, run_pkg_installer_from_disk_image};
use crate::error::FrabbitError;
use crate::operation::{PlannedExecutionKind, PlannedExecutionPlan};

pub fn execute_planned_execution(plan: &PlannedExecutionPlan, dry_run: bool) -> Result<()> {
    if dry_run {
        return Ok(());
    }

    match plan.kind {
        PlannedExecutionKind::LaunchInstallerExecutable => execute_program_plan(plan)?,
        PlannedExecutionKind::MountDiskImageAndCopyAppBundle => {
            execute_disk_image_app_bundle_plan(plan)?;
        }
        PlannedExecutionKind::ExtractArchiveAndCopyOsaraAssets => {
            execute_osara_archive_plan(plan)?;
        }
        PlannedExecutionKind::MountDiskImageAndRunPkgInstaller => {
            execute_disk_image_pkg_installer_plan(plan)?;
        }
        PlannedExecutionKind::ExtractArchiveAndRunInstaller
        | PlannedExecutionKind::MountDiskImageAndRunInstaller => {
            return Err(FrabbitError::InvalidPlannedExecution {
                message: format!("runner {:?} is not implemented yet", plan.kind),
            });
        }
    }

    Ok(())
}

fn execute_disk_image_pkg_installer_plan(plan: &PlannedExecutionPlan) -> Result<()> {
    let pkg_suffix =
        plan.arguments
            .first()
            .ok_or_else(|| FrabbitError::InvalidPlannedExecution {
                message: "disk-image pkg-installer plan did not provide a pkg filename suffix"
                    .to_string(),
            })?;
    run_pkg_installer_from_disk_image(Path::new(&plan.artifact_location), pkg_suffix)?;
    Ok(())
}

fn execute_osara_archive_plan(plan: &PlannedExecutionPlan) -> Result<()> {
    let resource_path =
        plan.arguments
            .first()
            .ok_or_else(|| FrabbitError::InvalidPlannedExecution {
                message: "OSARA archive plan did not provide a resource path".to_string(),
            })?;
    extract_osara_macos_assets(Path::new(&plan.artifact_location), Path::new(resource_path))?;
    Ok(())
}

fn execute_disk_image_app_bundle_plan(plan: &PlannedExecutionPlan) -> Result<()> {
    let bundle_basename =
        plan.arguments
            .first()
            .ok_or_else(|| FrabbitError::InvalidPlannedExecution {
                message: "disk-image app-bundle plan did not provide a bundle basename".to_string(),
            })?;
    let install_destination =
        plan.arguments
            .get(1)
            .ok_or_else(|| FrabbitError::InvalidPlannedExecution {
                message: "disk-image app-bundle plan did not provide an install destination"
                    .to_string(),
            })?;
    install_app_bundle_from_disk_image(
        Path::new(&plan.artifact_location),
        Path::new(install_destination),
        bundle_basename,
    )?;
    Ok(())
}

pub fn verify_planned_execution_paths(plan: &PlannedExecutionPlan) -> Result<()> {
    verify_paths(&plan.verification_paths)
}

/// Reject the run when any `freshness_paths` entry's mtime is older than
/// `install_started_at`. Catches "silent install no-op" cases where the
/// installer returned success but actually didn't write anything — the
/// regular existence-only [`verify_planned_execution_paths`] would still
/// pass because a previous install left the file on disk.
pub fn verify_planned_execution_freshness(
    plan: &PlannedExecutionPlan,
    install_started_at: SystemTime,
) -> Result<()> {
    let mut stale = Vec::new();
    for path in &plan.freshness_paths {
        match std::fs::metadata(path) {
            Ok(metadata) => match metadata.modified() {
                Ok(mtime) if mtime >= install_started_at => {}
                _ => stale.push(path.clone()),
            },
            Err(_) => stale.push(path.clone()),
        }
    }
    if stale.is_empty() {
        Ok(())
    } else {
        Err(FrabbitError::PostInstallVerificationFailed {
            missing_paths: stale,
        })
    }
}

fn execute_program_plan(plan: &PlannedExecutionPlan) -> Result<()> {
    let Some(program) = &plan.program else {
        return Err(FrabbitError::InvalidPlannedExecution {
            message: "launch plan did not provide a program path".to_string(),
        });
    };

    // Only raise a UAC prompt when we actually need to. If FRABBIT already
    // runs elevated — the user disabled UAC (an admin's processes then carry
    // the full token), or launched FRABBIT "as administrator" — the vendor
    // installer can be run directly. Invoking the `runas` verb in that state
    // is not just redundant: with UAC fully disabled there is no elevation
    // service, so `runas` fails with ERROR_CANCELLED (1223) and the install
    // looks "cancelled" though the user never saw a prompt.
    if plan.requires_elevation && !frabbit_platform::is_process_elevated() {
        return execute_program_plan_elevated(plan, program);
    }

    let mut command = Command::new(program);
    command.args(&plan.arguments);
    if let Some(working_directory) = &plan.working_directory {
        command.current_dir(working_directory);
    }

    let status = command.status().map_err(|source| FrabbitError::Io {
        path: PathBuf::from(program),
        source,
    })?;
    if !status.success() {
        // We are on the *direct* (non-`runas`) launch path: either the plan
        // didn't require elevation, or FRABBIT already runs elevated. A
        // `CreateProcess` launch never raises a UAC prompt (it fails with
        // ERROR_ELEVATION_REQUIRED, 740, instead), so a 1223 here is the
        // installer's *own* exit code — NOT a declined UAC prompt. Report
        // the real exit code honestly rather than mislabelling every 1223 as
        // "administrator approval cancelled", which hid the true failure.
        return Err(FrabbitError::ProcessFailed {
            program: program.clone(),
            exit_code: status.code(),
        });
    }

    Ok(())
}

fn execute_program_plan_elevated(plan: &PlannedExecutionPlan, program: &str) -> Result<()> {
    use frabbit_platform::ElevationError;

    let exit_code = frabbit_platform::run_elevated_and_wait(
        Path::new(program),
        &plan.arguments,
        plan.working_directory.as_deref(),
    )
    .map_err(|error| match error {
        ElevationError::UserCancelledElevation { .. } => FrabbitError::UserCancelledElevation {
            program: program.to_string(),
        },
        other => FrabbitError::InvalidPlannedExecution {
            message: other.to_string(),
        },
    })?;

    match exit_code {
        Some(0) => Ok(()),
        // Exit code 1223 is Windows `ERROR_CANCELLED`. Even after elevation
        // succeeds, an upstream installer returns it when the user cancels the
        // install (or dismisses a prompt it raised), so surface the same
        // recoverable "was cancelled — re-run and approve" message instead of
        // a raw "process failed with exit code Some(1223)".
        Some(1223) if cfg!(target_os = "windows") => Err(FrabbitError::UserCancelledElevation {
            program: program.to_string(),
        }),
        Some(code) => Err(FrabbitError::ProcessFailed {
            program: program.to_string(),
            exit_code: Some(code),
        }),
        None => Err(FrabbitError::ProcessFailed {
            program: program.to_string(),
            exit_code: None,
        }),
    }
}

fn verify_paths(paths: &[PathBuf]) -> Result<()> {
    let missing_paths = paths
        .iter()
        .filter(|path| !path.exists())
        .cloned()
        .collect::<Vec<_>>();
    if missing_paths.is_empty() {
        Ok(())
    } else {
        Err(FrabbitError::PostInstallVerificationFailed { missing_paths })
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{execute_planned_execution, verify_planned_execution_paths};
    use crate::operation::{PlannedExecutionKind, PlannedExecutionPlan};

    #[test]
    fn dry_run_does_not_execute_program() {
        let dir = tempdir().unwrap();
        let marker_path = dir.path().join("marker.txt");
        let plan = success_plan(&marker_path);

        execute_planned_execution(&plan, true).unwrap();

        assert!(!marker_path.exists());
    }

    #[test]
    fn executes_program_and_verifies_output() {
        let dir = tempdir().unwrap();
        let marker_path = dir.path().join("marker.txt");
        let plan = success_plan(&marker_path);

        execute_planned_execution(&plan, false).unwrap();
        verify_planned_execution_paths(&plan).unwrap();

        assert!(marker_path.is_file());
    }

    #[test]
    fn fails_when_program_returns_nonzero() {
        let dir = tempdir().unwrap();
        let marker_path = dir.path().join("marker.txt");
        let plan = failure_plan(&marker_path);

        let error = execute_planned_execution(&plan, false).unwrap_err();

        assert!(error.to_string().contains("process failed"));
    }

    #[test]
    fn verification_fails_when_expected_output_is_missing() {
        let dir = tempdir().unwrap();
        let marker_path = dir.path().join("missing.txt");
        let plan = PlannedExecutionPlan {
            kind: PlannedExecutionKind::LaunchInstallerExecutable,
            artifact_location: "noop".to_string(),
            program: None,
            arguments: Vec::new(),
            working_directory: None,
            verification_paths: vec![marker_path],
            requires_elevation: false,
            freshness_paths: Vec::new(),
        };

        let error = verify_planned_execution_paths(&plan).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("post-install verification failed")
        );
    }

    #[cfg(target_os = "windows")]
    fn success_plan(marker_path: &std::path::Path) -> PlannedExecutionPlan {
        PlannedExecutionPlan {
            kind: PlannedExecutionKind::LaunchInstallerExecutable,
            artifact_location: "powershell.exe".to_string(),
            program: Some("powershell.exe".to_string()),
            arguments: vec![
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-Command".to_string(),
                format!(
                    "Set-Content -Path '{}' -Value 'ok'",
                    escaped_path(marker_path)
                ),
            ],
            working_directory: None,
            verification_paths: vec![marker_path.to_path_buf()],
            requires_elevation: false,
            freshness_paths: Vec::new(),
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn success_plan(marker_path: &std::path::Path) -> PlannedExecutionPlan {
        PlannedExecutionPlan {
            kind: PlannedExecutionKind::LaunchInstallerExecutable,
            artifact_location: "sh".to_string(),
            program: Some("sh".to_string()),
            arguments: vec![
                "-c".to_string(),
                format!("printf ok > \"{}\"", escaped_path(marker_path)),
            ],
            working_directory: None,
            verification_paths: vec![marker_path.to_path_buf()],
            requires_elevation: false,
            freshness_paths: Vec::new(),
        }
    }

    #[cfg(target_os = "windows")]
    fn failure_plan(marker_path: &std::path::Path) -> PlannedExecutionPlan {
        PlannedExecutionPlan {
            kind: PlannedExecutionKind::LaunchInstallerExecutable,
            artifact_location: "powershell.exe".to_string(),
            program: Some("powershell.exe".to_string()),
            arguments: vec![
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-Command".to_string(),
                format!(
                    "Set-Content -Path '{}' -Value 'fail'; exit 7",
                    escaped_path(marker_path)
                ),
            ],
            working_directory: None,
            verification_paths: vec![marker_path.to_path_buf()],
            requires_elevation: false,
            freshness_paths: Vec::new(),
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn failure_plan(marker_path: &std::path::Path) -> PlannedExecutionPlan {
        PlannedExecutionPlan {
            kind: PlannedExecutionKind::LaunchInstallerExecutable,
            artifact_location: "sh".to_string(),
            program: Some("sh".to_string()),
            arguments: vec![
                "-c".to_string(),
                format!("printf fail > \"{}\"; exit 7", escaped_path(marker_path)),
            ],
            working_directory: None,
            verification_paths: vec![marker_path.to_path_buf()],
            requires_elevation: false,
            freshness_paths: Vec::new(),
        }
    }

    #[cfg(target_os = "windows")]
    fn escaped_path(path: &std::path::Path) -> String {
        path.display().to_string().replace('\'', "''")
    }

    #[cfg(not(target_os = "windows"))]
    fn escaped_path(path: &std::path::Path) -> String {
        path.display().to_string().replace('"', "\\\"")
    }
}
