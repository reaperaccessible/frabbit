use std::path::{Path, PathBuf};

use crate::artifact::ArtifactKind;
use crate::error::{FrabbitError, IoPathContext, Result};
use crate::model::Platform;
use crate::package::PACKAGE_OSARA;
use crate::reapack::extract_scr_lines;

use super::{
    KeymapChoice, PackageAutomationSupport, PlannedAutomationKind, PlannedExecutionKind,
    PlannedExecutionOverride, UnattendedPostInstallReport, backup_file_for_unattended_change,
    target_likely_portable,
};

pub(super) const TITLE: &str = "OSARA";

const RA_WIN_USA: &[u8] = include_bytes!(
    "../../../../Contents/KeyMaps_Win/KeyMap ReaperAccessible - Win - USA.ReaperKeyMap"
);
const RA_WIN_FRF: &[u8] = include_bytes!(
    "../../../../Contents/KeyMaps_Win/KeyMap ReaperAccessible - Win - FRF.ReaperKeyMap"
);
const RA_WIN_FRC: &[u8] = include_bytes!(
    "../../../../Contents/KeyMaps_Win/KeyMap ReaperAccessible - Win - FRC.ReaperKeyMap"
);
const RA_MAC_USA: &[u8] = include_bytes!(
    "../../../../Contents/KeyMaps_Mac/KeyMap ReaperAccessible - Mac - USA.ReaperKeyMap"
);
const RA_MAC_FRF: &[u8] = include_bytes!(
    "../../../../Contents/KeyMaps_Mac/KeyMap ReaperAccessible - Mac - FRF.ReaperKeyMap"
);
const RA_MAC_FRC: &[u8] = include_bytes!(
    "../../../../Contents/KeyMaps_Mac/KeyMap ReaperAccessible - Mac - FRC.ReaperKeyMap"
);

pub(crate) fn embedded_keymap_bytes(choice: KeymapChoice) -> Option<&'static [u8]> {
    match choice {
        KeymapChoice::ReaperAccessibleWinUsa => Some(RA_WIN_USA),
        KeymapChoice::ReaperAccessibleWinFrf => Some(RA_WIN_FRF),
        KeymapChoice::ReaperAccessibleWinFrc => Some(RA_WIN_FRC),
        KeymapChoice::ReaperAccessibleMacUsa => Some(RA_MAC_USA),
        KeymapChoice::ReaperAccessibleMacFrf => Some(RA_MAC_FRF),
        KeymapChoice::ReaperAccessibleMacFrc => Some(RA_MAC_FRC),
        _ => None,
    }
}

/// OSARA-specific automation routing. Today: Windows installer is unattended,
/// macOS archive is unattended via the OSARA-asset extractor.
pub(super) fn automation_support_for(
    kind: ArtifactKind,
    platform: Platform,
) -> Option<PackageAutomationSupport> {
    match (kind, platform) {
        (ArtifactKind::Installer, Platform::Windows) => Some(
            PackageAutomationSupport::AvailableUnattended(PlannedAutomationKind::VendorInstaller),
        ),
        (ArtifactKind::Archive, Platform::MacOs) => Some(
            PackageAutomationSupport::AvailableUnattended(PlannedAutomationKind::ArchiveExtraction),
        ),
        _ => None,
    }
}

/// OSARA-specific message variant used when the unattended path also applied
/// the key-map replacement step. Returns `None` when the replacement was not
/// requested (caller should fall back to the generic message). The pair
/// is (English text for the saved JSON report, structured code for the
/// localizable UI surface).
pub(super) fn unattended_install_message(
    keymap_choice: KeymapChoice,
    keymap_was_backed_up: bool,
) -> Option<(String, super::PackageOperationMessage)> {
    if !keymap_choice.replaces_keymap() {
        return None;
    }
    Some(if keymap_was_backed_up {
        (
            "FRABBIT ran the upstream installer unattended, backed up reaper-kb.ini, applied the key map replacement, and updated the FRABBIT receipt.".to_string(),
            super::PackageOperationMessage::OsaraUnattendedInstalledKeymapBackedUp,
        )
    } else {
        (
            "FRABBIT ran the upstream installer unattended, applied the key map replacement, and updated the FRABBIT receipt.".to_string(),
            super::PackageOperationMessage::OsaraUnattendedInstalledKeymapReplaced,
        )
    })
}

pub(super) fn manual_install_notes(
    resource_path: &Path,
    keymap_choice: KeymapChoice,
) -> Vec<String> {
    let mut notes = vec![
        "OSARA's Windows installer supports standard and portable REAPER targets; preserve an existing key map unless the user explicitly chooses replacement."
            .to_string(),
    ];
    if keymap_choice.replaces_keymap() {
        notes.push(format!(
            "The selected workflow replaces the current key map. Back up {} before replacing it.",
            resource_path.join("reaper-kb.ini").display()
        ));
    } else {
        notes.push(format!(
            "The selected workflow preserves the current key map. Leave {} unchanged.",
            resource_path.join("reaper-kb.ini").display()
        ));
    }
    notes
}

/// Files installed by OSARA that the receipt should reference. Filtered to
/// the on-disk existing ones after the unattended run.
pub(super) fn receipt_paths(resource_path: &Path, keymap_choice: KeymapChoice) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let keymap_path = resource_path.join("KeyMaps").join("OSARA.ReaperKeyMap");
    if keymap_path.exists() {
        paths.push(keymap_path);
    }
    let support_dir = resource_path.join("osara");
    if support_dir.exists() {
        paths.push(support_dir);
    }
    if keymap_choice.replaces_keymap() {
        let current_keymap = resource_path.join("reaper-kb.ini");
        if current_keymap.exists() {
            paths.push(current_keymap);
        }
    }
    paths
}

/// Post-install fixups specific to OSARA: clean up the portable
/// uninstaller stub on Windows. Keymap replacement is now handled
/// independently in setup.rs after all packages are installed.
pub(super) fn post_install_unattended(
    resource_path: &Path,
    platform: Platform,
    target_app_path: Option<&Path>,
    _keymap_choice: KeymapChoice,
) -> Result<UnattendedPostInstallReport> {
    let report = UnattendedPostInstallReport::default();
    if matches!(platform, Platform::Windows)
        && target_likely_portable(resource_path, target_app_path)
    {
        let uninstall_path = resource_path.join("osara").join("uninstall.exe");
        if uninstall_path.is_file() {
            std::fs::remove_file(&uninstall_path).with_path(&uninstall_path)?;
        }
    }
    Ok(report)
}

pub(super) fn verification_paths(
    resource_path: &Path,
    _keymap_choice: KeymapChoice,
) -> Vec<PathBuf> {
    vec![
        resource_path.join("UserPlugins"),
        resource_path.join("KeyMaps").join("OSARA.ReaperKeyMap"),
        resource_path.join("osara"),
    ]
}

pub(super) fn installer_arguments(
    kind: ArtifactKind,
    platform: Platform,
    resource_path: &Path,
) -> Option<Vec<String>> {
    match (kind, platform) {
        (ArtifactKind::Installer, Platform::Windows) => {
            Some(osara_windows_installer_arguments(resource_path))
        }
        _ => None,
    }
}

pub(super) fn planned_execution_override(
    kind: ArtifactKind,
    platform: Platform,
    resource_path: &Path,
) -> Option<PlannedExecutionOverride> {
    match (kind, platform) {
        (ArtifactKind::Archive, Platform::MacOs) => Some(PlannedExecutionOverride {
            kind: PlannedExecutionKind::ExtractArchiveAndCopyOsaraAssets,
            arguments: vec![resource_path.display().to_string()],
            use_cached_working_dir: true,
        }),
        _ => None,
    }
}

fn osara_windows_installer_arguments(resource_path: &Path) -> Vec<String> {
    vec!["/S".to_string(), format!("/D={}", resource_path.display())]
}

pub(super) fn osara_manual_steps(
    kind: ArtifactKind,
    resource_path: &Path,
    keymap_choice: KeymapChoice,
) -> Vec<String> {
    let mut steps = match kind {
        ArtifactKind::Installer => vec![format!(
            "When the OSARA installer asks for the REAPER target, choose this resource or portable folder: {}",
            resource_path.display()
        )],
        ArtifactKind::Archive | ArtifactKind::SevenZipArchive => vec![format!(
            "Run the OSARA installer from the extracted archive and target this REAPER resource or portable folder: {}",
            resource_path.display()
        )],
        ArtifactKind::DiskImage => vec![format!(
            "Run the OSARA installer from the opened disk image and target this REAPER resource or portable folder: {}",
            resource_path.display()
        )],
        ArtifactKind::ExtensionBinary => vec![format!(
            "Copy the OSARA extension into this REAPER UserPlugins folder: {}",
            resource_path.join("UserPlugins").display()
        )],
    };
    if keymap_choice.replaces_keymap() {
        steps.push(format!(
            "After backing up {}, replace the current key map.",
            resource_path.join("reaper-kb.ini").display()
        ));
    } else {
        steps.push(
            "Preserve the current key map if the OSARA installer offers a replacement option."
                .to_string(),
        );
    }
    steps
}

/// Returns the file name of the .ReaperKeyMap backup that should be
/// created in <resource>/KeyMaps/ when this KeymapChoice's keymap
/// replaces the user's current reaper-kb.ini. None for PreserveCurrent.
fn replaced_backup_filename(choice: KeymapChoice) -> Option<&'static str> {
    match choice {
        KeymapChoice::PreserveCurrent => None,
        KeymapChoice::Osara => Some("OSARAReplacedBackup.ReaperKeyMap"),
        KeymapChoice::ReaperAccessibleWinUsa | KeymapChoice::ReaperAccessibleMacUsa => {
            Some("ReaperAccessibleUSAReplacedBackup.ReaperKeyMap")
        }
        KeymapChoice::ReaperAccessibleWinFrf | KeymapChoice::ReaperAccessibleMacFrf => {
            Some("ReaperAccessibleFRFRReplacedBackup.ReaperKeyMap")
        }
        KeymapChoice::ReaperAccessibleWinFrc | KeymapChoice::ReaperAccessibleMacFrc => {
            Some("ReaperAccessibleFRCAReplacedBackup.ReaperKeyMap")
        }
    }
}

/// Returns the file name of the .ReaperKeyMap that should also be
/// installed permanently in <resource>/KeyMaps/ as a reference copy.
/// None for PreserveCurrent.
fn keymap_reference_filename(choice: KeymapChoice) -> Option<&'static str> {
    match choice {
        KeymapChoice::PreserveCurrent => None,
        KeymapChoice::Osara => Some("OSARA.ReaperKeyMap"),
        KeymapChoice::ReaperAccessibleWinUsa | KeymapChoice::ReaperAccessibleMacUsa => {
            Some("ReaperAccessibleUSA.ReaperKeyMap")
        }
        KeymapChoice::ReaperAccessibleWinFrf | KeymapChoice::ReaperAccessibleMacFrf => {
            Some("ReaperAccessibleFRFR.ReaperKeyMap")
        }
        KeymapChoice::ReaperAccessibleWinFrc | KeymapChoice::ReaperAccessibleMacFrc => {
            Some("ReaperAccessibleFRCA.ReaperKeyMap")
        }
    }
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub(crate) struct KeymapApplyReport {
    pub active_path: PathBuf,
    pub reference_path: Option<PathBuf>,
    pub backup_path: Option<PathBuf>,
}

/// Replicate OSARA's NSIS installer keymap behavior for a chosen variant.
///
/// Steps (from osara/installer/osara.nsi):
/// 1. Always write the reference copy to <resource>/KeyMaps/<variant>.ReaperKeyMap
/// 2. Delete any pre-existing backup at <resource>/KeyMaps/<variant>ReplacedBackup.ReaperKeyMap
/// 3. If <resource>/reaper-kb.ini exists, RENAME it (move, not copy) to that backup path
/// 4. Write the new <resource>/reaper-kb.ini with the chosen keymap content
fn apply_keymap_osara_style(
    resource_path: &Path,
    choice: KeymapChoice,
    keymap_bytes: &[u8],
) -> Result<KeymapApplyReport> {
    let keymaps_dir = resource_path.join("KeyMaps");
    std::fs::create_dir_all(&keymaps_dir).with_path(&keymaps_dir)?;

    // Step 1: reference copy
    let reference_path = if let Some(ref_name) = keymap_reference_filename(choice) {
        let ref_path = keymaps_dir.join(ref_name);
        std::fs::write(&ref_path, keymap_bytes).with_path(&ref_path)?;
        Some(ref_path)
    } else {
        None
    };

    let mut backup_path: Option<PathBuf> = None;
    if let Some(backup_name) = replaced_backup_filename(choice) {
        let backup_target = keymaps_dir.join(backup_name);
        // Step 2: delete previous backup
        if backup_target.exists() {
            std::fs::remove_file(&backup_target).with_path(&backup_target)?;
        }
        // Step 3: rename current reaper-kb.ini -> backup
        let active = resource_path.join("reaper-kb.ini");
        if active.is_file() {
            std::fs::rename(&active, &backup_target).with_path(&backup_target)?;
            backup_path = Some(backup_target);
        }
    }

    // Step 4: write new active reaper-kb.ini
    let new_active = resource_path.join("reaper-kb.ini");
    std::fs::write(&new_active, keymap_bytes).with_path(&new_active)?;

    Ok(KeymapApplyReport {
        active_path: new_active,
        reference_path,
        backup_path,
    })
}

pub(crate) fn apply_osara_keymap_replacement(
    resource_path: &Path,
) -> Result<UnattendedPostInstallReport> {
    let replacement_source = resource_path.join("KeyMaps").join("OSARA.ReaperKeyMap");
    if !replacement_source.is_file() {
        return Err(FrabbitError::PostInstallVerificationFailed {
            missing_paths: vec![replacement_source],
        });
    }

    let current_keymap = resource_path.join("reaper-kb.ini");
    let mut report = UnattendedPostInstallReport::default();
    let mut preserved_scr_lines: Vec<String> = Vec::new();

    if current_keymap.is_file() {
        // Capture the existing SCR records before overwriting. ReaPack
        // registers every installed ReaScript through these lines (via
        // REAPER's `AddRemoveReaScript` API); a plain overwrite would
        // wipe them, so installed packages would silently vanish from
        // REAPER's actions list until the user manually ran
        // "ReaPack: Synchronize packages" inside REAPER. Re-appending
        // the lines after writing OSARA's key map sidesteps that step.
        let raw = std::fs::read(&current_keymap).with_path(&current_keymap)?;
        let existing = String::from_utf8_lossy(&raw);
        preserved_scr_lines = extract_scr_lines(&existing);

        let (backup_path, backup_manifest_path) = backup_file_for_unattended_change(
            resource_path,
            PACKAGE_OSARA,
            &current_keymap,
            "osara-keymap-replacement",
        )?;
        report.backup_paths.push(backup_path);
        report.backup_manifest_path = Some(backup_manifest_path);
    }

    // Read replacement bytes then apply OSARA-style flow (reference,
    // delete-old-backup, rename current to backup, write new active).
    let keymap_bytes = std::fs::read(&replacement_source).with_path(&replacement_source)?;
    apply_keymap_osara_style(resource_path, KeymapChoice::Osara, &keymap_bytes)?;

    if !preserved_scr_lines.is_empty() {
        append_lines_preserving_newline(&current_keymap, &preserved_scr_lines)?;
    }

    Ok(report)
}

pub(crate) fn apply_keymap_from_bytes(
    resource_path: &Path,
    keymap_bytes: &[u8],
    choice: KeymapChoice,
) -> Result<UnattendedPostInstallReport> {
    let current_keymap = resource_path.join("reaper-kb.ini");
    let mut report = UnattendedPostInstallReport::default();
    let mut preserved_scr_lines: Vec<String> = Vec::new();

    if current_keymap.is_file() {
        let raw = std::fs::read(&current_keymap).with_path(&current_keymap)?;
        let existing = String::from_utf8_lossy(&raw);
        preserved_scr_lines = extract_scr_lines(&existing);

        let (backup_path, backup_manifest_path) = backup_file_for_unattended_change(
            resource_path,
            "reaper-accessible-keymap",
            &current_keymap,
            "reaper-accessible-keymap-replacement",
        )?;
        report.backup_paths.push(backup_path);
        report.backup_manifest_path = Some(backup_manifest_path);
    }

    // OSARA-style: write reference into KeyMaps/, rename current
    // reaper-kb.ini to <variant>ReplacedBackup.ReaperKeyMap (deleting
    // any old backup first), then write the new active reaper-kb.ini.
    apply_keymap_osara_style(resource_path, choice, keymap_bytes)?;

    if !preserved_scr_lines.is_empty() {
        append_lines_preserving_newline(&current_keymap, &preserved_scr_lines)?;
    }

    Ok(report)
}

fn append_lines_preserving_newline(target_path: &Path, lines: &[String]) -> Result<()> {
    let raw = std::fs::read(target_path).with_path(target_path)?;
    let existing = String::from_utf8_lossy(&raw).into_owned();
    let newline = if existing.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut out = existing;
    if !out.is_empty() && !out.ends_with(newline) {
        out.push_str(newline);
    }
    for line in lines {
        out.push_str(line);
        out.push_str(newline);
    }
    std::fs::write(target_path, out).with_path(target_path)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::apply_osara_keymap_replacement;

    fn seed_osara_replacement_source(resource_path: &std::path::Path, body: &str) {
        let keymaps = resource_path.join("KeyMaps");
        fs::create_dir_all(&keymaps).unwrap();
        fs::write(keymaps.join("OSARA.ReaperKeyMap"), body).unwrap();
    }

    #[test]
    fn preserves_existing_scr_lines_when_replacing_keymap() {
        let dir = tempdir().unwrap();
        let resource_path = dir.path();
        seed_osara_replacement_source(resource_path, "osara keymap\r\n");

        let existing = "ACT 1 0 \"_RSabc\" \"Custom\" _SWS_ABOUT\r\n\
                        KEY 9 65 _RSabc 0\r\n\
                        SCR 4 0 RSdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef \"Script: foo.lua\" foo.lua\r\n\
                        SCR 260 32060 RScafef00d \"Script: midi.lua\" midi.lua\r\n";
        fs::write(resource_path.join("reaper-kb.ini"), existing).unwrap();

        let report = apply_osara_keymap_replacement(resource_path).unwrap();

        let new_contents = fs::read_to_string(resource_path.join("reaper-kb.ini")).unwrap();
        assert!(new_contents.starts_with("osara keymap\r\n"));
        assert!(new_contents.contains(
            "SCR 4 0 RSdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef \"Script: foo.lua\" foo.lua"
        ));
        assert!(new_contents.contains("SCR 260 32060 RScafef00d \"Script: midi.lua\" midi.lua"));
        // Non-SCR records from the prior key map must NOT come back —
        // OSARA's replacement is intentionally an authoritative rewrite of
        // KEY / ACT bindings.
        assert!(!new_contents.contains("ACT 1 0"));
        assert!(!new_contents.contains("KEY 9 65"));

        // Backup was created since an existing reaper-kb.ini was present.
        assert_eq!(report.backup_paths.len(), 1);
        assert_eq!(
            fs::read_to_string(&report.backup_paths[0]).unwrap(),
            existing
        );
    }

    #[test]
    fn replacement_is_clean_when_old_keymap_has_no_scr_lines() {
        let dir = tempdir().unwrap();
        let resource_path = dir.path();
        seed_osara_replacement_source(resource_path, "osara keymap\r\n");
        fs::write(
            resource_path.join("reaper-kb.ini"),
            "ACT 1 0 \"_RSabc\" \"Custom\" _SWS_ABOUT\r\nKEY 9 65 _RSabc 0\r\n",
        )
        .unwrap();

        apply_osara_keymap_replacement(resource_path).unwrap();

        assert_eq!(
            fs::read_to_string(resource_path.join("reaper-kb.ini")).unwrap(),
            "osara keymap\r\n"
        );
    }

    #[test]
    fn replacement_when_no_prior_keymap_writes_only_osara_content() {
        let dir = tempdir().unwrap();
        let resource_path = dir.path();
        seed_osara_replacement_source(resource_path, "osara keymap\r\n");

        let report = apply_osara_keymap_replacement(resource_path).unwrap();

        assert_eq!(
            fs::read_to_string(resource_path.join("reaper-kb.ini")).unwrap(),
            "osara keymap\r\n"
        );
        assert!(report.backup_paths.is_empty());
        assert!(report.backup_manifest_path.is_none());
    }

    #[test]
    fn osara_keymap_backup_goes_to_keymaps_folder() {
        let dir = tempdir().unwrap();
        let resource_path = dir.path();
        seed_osara_replacement_source(resource_path, "osara keymap\r\n");
        fs::write(resource_path.join("reaper-kb.ini"), "ORIGINAL CONTENT\r\n").unwrap();

        apply_osara_keymap_replacement(resource_path).unwrap();

        // OSARA-style backup lives inside KeyMaps/ with a variant-specific name.
        let backup = resource_path
            .join("KeyMaps")
            .join("OSARAReplacedBackup.ReaperKeyMap");
        assert!(
            backup.is_file(),
            "expected KeyMaps/OSARAReplacedBackup.ReaperKeyMap"
        );
        assert_eq!(fs::read_to_string(&backup).unwrap(), "ORIGINAL CONTENT\r\n");
        // Old-style sibling backup must NOT be created any more.
        assert!(!resource_path.join("reaper-kb.ini.bak").exists());
        // And the active reaper-kb.ini now holds the new keymap.
        assert!(
            fs::read_to_string(resource_path.join("reaper-kb.ini"))
                .unwrap()
                .starts_with("osara keymap")
        );
    }

    #[test]
    fn no_keymaps_backup_when_reaper_kb_ini_is_absent() {
        let dir = tempdir().unwrap();
        let resource_path = dir.path();
        seed_osara_replacement_source(resource_path, "osara keymap\r\n");

        apply_osara_keymap_replacement(resource_path).unwrap();

        // Fresh REAPER install: nothing to rename, so no backup file.
        assert!(
            !resource_path
                .join("KeyMaps")
                .join("OSARAReplacedBackup.ReaperKeyMap")
                .exists()
        );
        // The reference copy is still installed in KeyMaps/.
        assert!(
            resource_path
                .join("KeyMaps")
                .join("OSARA.ReaperKeyMap")
                .is_file()
        );
        // The new active keymap is still written.
        assert_eq!(
            fs::read_to_string(resource_path.join("reaper-kb.ini")).unwrap(),
            "osara keymap\r\n"
        );
    }

    #[test]
    fn keymaps_backup_is_replaced_on_second_run() {
        let dir = tempdir().unwrap();
        let resource_path = dir.path();
        seed_osara_replacement_source(resource_path, "osara keymap\r\n");

        // First run: V1 becomes the backup.
        fs::write(resource_path.join("reaper-kb.ini"), "V1\r\n").unwrap();
        apply_osara_keymap_replacement(resource_path).unwrap();
        let backup = resource_path
            .join("KeyMaps")
            .join("OSARAReplacedBackup.ReaperKeyMap");
        assert_eq!(fs::read_to_string(&backup).unwrap(), "V1\r\n");

        // Second run: OSARA-style behavior deletes the old backup and
        // replaces it with whatever reaper-kb.ini currently holds — so
        // the freshly-applied keymap from the first run becomes the
        // new backup. Matches OSARA's NSIS installer semantics.
        // Seed a fresh source so the test stays deterministic.
        seed_osara_replacement_source(resource_path, "osara keymap v2\r\n");
        apply_osara_keymap_replacement(resource_path).unwrap();
        assert_eq!(
            fs::read_to_string(&backup).unwrap(),
            "osara keymap\r\n",
            "second run must replace the backup with the previously-active keymap"
        );
    }

    #[test]
    fn reaper_accessible_keymap_uses_variant_specific_backup_name() {
        use super::{KeymapChoice, apply_keymap_from_bytes};
        let dir = tempdir().unwrap();
        let resource_path = dir.path();
        fs::write(resource_path.join("reaper-kb.ini"), "RA ORIGINAL\r\n").unwrap();

        apply_keymap_from_bytes(
            resource_path,
            b"new ra keymap\r\n",
            KeymapChoice::ReaperAccessibleWinUsa,
        )
        .unwrap();

        let backup = resource_path
            .join("KeyMaps")
            .join("ReaperAccessibleUSAReplacedBackup.ReaperKeyMap");
        assert!(
            backup.is_file(),
            "expected variant-specific backup name in KeyMaps/"
        );
        assert_eq!(fs::read_to_string(&backup).unwrap(), "RA ORIGINAL\r\n");
    }

    #[test]
    fn osara_keymap_writes_reference_copy_in_keymaps_folder() {
        let dir = tempdir().unwrap();
        let resource_path = dir.path();
        seed_osara_replacement_source(resource_path, "osara keymap\r\n");
        fs::write(resource_path.join("reaper-kb.ini"), "OLD\r\n").unwrap();

        apply_osara_keymap_replacement(resource_path).unwrap();

        let reference = resource_path.join("KeyMaps").join("OSARA.ReaperKeyMap");
        assert!(reference.is_file(), "KeyMaps/OSARA.ReaperKeyMap must exist");
    }

    #[test]
    fn each_variant_uses_its_own_backup_name() {
        use super::{KeymapChoice, apply_keymap_from_bytes};
        let cases: &[(KeymapChoice, &str)] = &[
            (
                KeymapChoice::ReaperAccessibleWinUsa,
                "ReaperAccessibleUSAReplacedBackup.ReaperKeyMap",
            ),
            (
                KeymapChoice::ReaperAccessibleWinFrf,
                "ReaperAccessibleFRFRReplacedBackup.ReaperKeyMap",
            ),
            (
                KeymapChoice::ReaperAccessibleWinFrc,
                "ReaperAccessibleFRCAReplacedBackup.ReaperKeyMap",
            ),
        ];
        for (choice, expected_name) in cases {
            let dir = tempdir().unwrap();
            let resource_path = dir.path();
            fs::write(resource_path.join("reaper-kb.ini"), "ORIG\r\n").unwrap();
            apply_keymap_from_bytes(resource_path, b"new\r\n", *choice).unwrap();
            assert!(
                resource_path.join("KeyMaps").join(expected_name).is_file(),
                "expected backup {expected_name} for {choice:?}"
            );
        }
    }

    #[test]
    fn preserves_scr_lines_using_lf_newline_when_osara_keymap_is_lf() {
        let dir = tempdir().unwrap();
        let resource_path = dir.path();
        // LF-only replacement source — exercise the newline detection path.
        seed_osara_replacement_source(resource_path, "osara keymap\n");
        fs::write(
            resource_path.join("reaper-kb.ini"),
            "SCR 4 0 RSdeadbeef \"Script: foo.lua\" foo.lua\n",
        )
        .unwrap();

        apply_osara_keymap_replacement(resource_path).unwrap();

        let new_contents = fs::read_to_string(resource_path.join("reaper-kb.ini")).unwrap();
        assert!(!new_contents.contains("\r\n"));
        assert_eq!(
            new_contents,
            "osara keymap\nSCR 4 0 RSdeadbeef \"Script: foo.lua\" foo.lua\n"
        );
    }
}
