use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;

use crate::Result;
use crate::error::FrabbitError;
use crate::model::{Architecture, Platform};
use crate::version::Version;

const USER_AGENT: &str = concat!(
    "FRABBIT/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/ReaperAccessible/frabbit)"
);

pub const DEFAULT_SELF_UPDATE_MANIFEST_URL: &str = "https://github.com/ReaperAccessible/frabbit/releases/latest/download/frabbit-update-stable.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfUpdateManifest {
    pub version: Version,
    pub channel: String,
    pub published_at: String,
    pub release_notes_url: Option<String>,
    pub minimum_supported_previous_version: Option<Version>,
    pub assets: SelfUpdateAssets,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfUpdateAssets {
    pub windows: Option<SelfUpdateAsset>,
    pub macos: Option<SelfUpdateAsset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platforms: Option<BTreeMap<String, SelfUpdateAsset>>,
    /// Zipped `.app` bundles, keyed like `platforms` (`macos-aarch64`). A
    /// macOS FRABBIT running from `Frabbit.app` updates from these rather than
    /// from the bare binary in `platforms`: the bundle also carries the version
    /// in `Info.plist`, the `.lproj` layout VoiceOver reads, and the code
    /// signature that seals them, none of which a binary swap would refresh.
    ///
    /// Deliberately a sibling field rather than extra `platforms` keys:
    /// [`validate_platform_key`] rejects any key that isn't `<os>-<arch>`, so
    /// a `macos-aarch64-bundle` entry would make every already-released client
    /// fail to parse the manifest at all — Windows ones included. Clients that
    /// predate this field ignore it and keep updating from `platforms`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundles: Option<BTreeMap<String, SelfUpdateAsset>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfUpdateAsset {
    pub url: String,
    pub sha256: String,
}

/// What the selected asset *is*, which decides how it gets installed: an
/// executable is copied over the running binary, an app bundle replaces the
/// whole `.app` directory it was launched from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SelfUpdateAssetKind {
    #[default]
    Executable,
    AppBundle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfUpdateAssetSelection {
    pub platform: Platform,
    pub url: String,
    pub sha256: String,
    /// Defaulted so reports serialized by older FRABBIT versions still load.
    #[serde(default)]
    pub kind: SelfUpdateAssetKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfUpdateCheckReport {
    pub manifest_url: String,
    pub current_version: Version,
    pub latest_version: Version,
    pub channel: String,
    pub published_at: String,
    pub release_notes_url: Option<String>,
    pub minimum_supported_previous_version: Option<Version>,
    pub update_available: bool,
    pub requires_manual_transition: bool,
    pub asset: SelfUpdateAssetSelection,
}

#[derive(Debug, Deserialize)]
struct RawSelfUpdateManifest {
    version: String,
    channel: String,
    published_at: String,
    release_notes_url: Option<String>,
    minimum_supported_previous_version: Option<String>,
    assets: RawSelfUpdateAssets,
}

#[derive(Debug, Deserialize)]
struct RawSelfUpdateAssets {
    windows: Option<RawSelfUpdateAsset>,
    macos: Option<RawSelfUpdateAsset>,
    #[serde(default)]
    platforms: Option<BTreeMap<String, RawSelfUpdateAsset>>,
    #[serde(default)]
    bundles: Option<BTreeMap<String, RawSelfUpdateAsset>>,
}

#[derive(Debug, Deserialize)]
struct RawSelfUpdateAsset {
    url: String,
    sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SemanticVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

pub fn current_frabbit_version() -> Result<Version> {
    parse_semantic_version(
        env!("CARGO_PKG_VERSION"),
        "build-metadata",
        "current_version",
    )
}

pub fn fetch_self_update_manifest(manifest_url: &str) -> Result<SelfUpdateManifest> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|source| FrabbitError::Http {
            url: "client-builder".to_string(),
            source,
        })?;

    let body = client
        .get(manifest_url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|source| FrabbitError::Http {
            url: manifest_url.to_string(),
            source,
        })?
        .text()
        .map_err(|source| FrabbitError::Http {
            url: manifest_url.to_string(),
            source,
        })?;

    parse_self_update_manifest(&body, manifest_url)
}

pub fn parse_self_update_manifest(body: &str, manifest_url: &str) -> Result<SelfUpdateManifest> {
    let raw: RawSelfUpdateManifest =
        serde_json::from_str(body).map_err(|source| FrabbitError::RemoteData {
            url: manifest_url.to_string(),
            message: source.to_string(),
        })?;

    let version = parse_semantic_version(&raw.version, manifest_url, "version")?;
    let minimum_supported_previous_version = raw
        .minimum_supported_previous_version
        .as_deref()
        .map(|value| {
            parse_semantic_version(value, manifest_url, "minimum_supported_previous_version")
        })
        .transpose()?;
    let platforms = parse_keyed_assets(raw.assets.platforms.as_ref(), manifest_url)?;
    let bundles = parse_keyed_assets(raw.assets.bundles.as_ref(), manifest_url)?;
    let assets = SelfUpdateAssets {
        windows: raw
            .assets
            .windows
            .as_ref()
            .map(|asset| parse_asset(asset, manifest_url, "windows"))
            .transpose()?,
        macos: raw
            .assets
            .macos
            .as_ref()
            .map(|asset| parse_asset(asset, manifest_url, "macos"))
            .transpose()?,
        platforms,
        bundles,
    };

    Ok(SelfUpdateManifest {
        version,
        channel: raw.channel,
        published_at: raw.published_at,
        release_notes_url: raw.release_notes_url,
        minimum_supported_previous_version,
        assets,
    })
}

pub fn check_self_update(platform: Platform, manifest_url: &str) -> Result<SelfUpdateCheckReport> {
    check_self_update_for(platform, manifest_url, SelfUpdateAssetKind::Executable)
}

/// [`check_self_update`], but choosing which flavour of asset the report should
/// point at. macOS FRABBIT passes [`SelfUpdateAssetKind::AppBundle`] when it is
/// running from `Frabbit.app`; everything else (the CLI, a bare binary) keeps
/// the executable asset.
pub fn check_self_update_for(
    platform: Platform,
    manifest_url: &str,
    preferred_kind: SelfUpdateAssetKind,
) -> Result<SelfUpdateCheckReport> {
    let manifest = fetch_self_update_manifest(manifest_url)?;
    evaluate_self_update_report(
        platform,
        Architecture::current(),
        manifest_url,
        current_frabbit_version()?,
        &manifest,
        preferred_kind,
    )
}

/// Download the update asset selected by [`check_self_update`] into
/// `dest_dir`, verify its SHA-256 against the manifest, and return the path to
/// the verified file. Bytes are streamed to a `.part` sibling and only renamed
/// into place once the checksum matches, so a corrupt or interrupted download
/// never leaves a runnable-looking file behind.
///
/// This does NOT replace the running executable or relaunch — it only produces
/// a verified copy of the new version on disk (that is Phase 3+).
pub fn download_and_verify_update(
    asset: &SelfUpdateAssetSelection,
    dest_dir: &std::path::Path,
) -> Result<std::path::PathBuf> {
    use std::io::Write;

    if !asset.url.starts_with("https://") {
        return Err(FrabbitError::RemoteData {
            url: asset.url.clone(),
            message: "update asset URL must use https".to_string(),
        });
    }

    std::fs::create_dir_all(dest_dir).map_err(|source| FrabbitError::Io {
        path: dest_dir.to_path_buf(),
        source,
    })?;

    let file_name = asset
        .url
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or("frabbit-update.exe");
    let final_path = dest_dir.join(file_name);
    let part_path = dest_dir.join(format!("{file_name}.part"));
    let _ = std::fs::remove_file(&part_path);

    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|source| FrabbitError::Http {
            url: asset.url.clone(),
            source,
        })?;
    let mut response = client
        .get(&asset.url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|source| FrabbitError::Http {
            url: asset.url.clone(),
            source,
        })?;

    let mut file = std::fs::File::create(&part_path).map_err(|source| FrabbitError::Io {
        path: part_path.clone(),
        source,
    })?;
    std::io::copy(&mut response, &mut file).map_err(|source| FrabbitError::Io {
        path: part_path.clone(),
        source,
    })?;
    file.flush().map_err(|source| FrabbitError::Io {
        path: part_path.clone(),
        source,
    })?;
    drop(file);

    let actual = crate::hash::sha256_file(&part_path)?;
    if !actual.eq_ignore_ascii_case(&asset.sha256) {
        let _ = std::fs::remove_file(&part_path);
        return Err(FrabbitError::RemoteData {
            url: asset.url.clone(),
            message: format!(
                "downloaded update failed checksum verification (expected {}, got {actual})",
                asset.sha256
            ),
        });
    }

    let _ = std::fs::remove_file(&final_path);
    std::fs::rename(&part_path, &final_path).map_err(|source| FrabbitError::Io {
        path: final_path.clone(),
        source,
    })?;
    Ok(final_path)
}

fn evaluate_self_update_report(
    platform: Platform,
    architecture: Architecture,
    manifest_url: &str,
    current_version: Version,
    manifest: &SelfUpdateManifest,
    preferred_kind: SelfUpdateAssetKind,
) -> Result<SelfUpdateCheckReport> {
    let current_semver =
        semantic_version_from_version(&current_version, manifest_url, "current_version")?;
    let latest_semver = semantic_version_from_version(&manifest.version, manifest_url, "version")?;
    let minimum_supported_previous_version = manifest.minimum_supported_previous_version.clone();
    let requires_manual_transition = minimum_supported_previous_version
        .as_ref()
        .map(|minimum| {
            semantic_version_from_version(
                minimum,
                manifest_url,
                "minimum_supported_previous_version",
            )
            .map(|minimum| current_semver < minimum)
        })
        .transpose()?
        .unwrap_or(false);

    Ok(SelfUpdateCheckReport {
        manifest_url: manifest_url.to_string(),
        current_version,
        latest_version: manifest.version.clone(),
        channel: manifest.channel.clone(),
        published_at: manifest.published_at.clone(),
        release_notes_url: manifest.release_notes_url.clone(),
        minimum_supported_previous_version,
        update_available: latest_semver > current_semver,
        requires_manual_transition,
        asset: select_asset_for_platform(
            platform,
            architecture,
            manifest,
            manifest_url,
            preferred_kind,
        )?,
    })
}

fn select_asset_for_platform(
    platform: Platform,
    architecture: Architecture,
    manifest: &SelfUpdateManifest,
    manifest_url: &str,
    preferred_kind: SelfUpdateAssetKind,
) -> Result<SelfUpdateAssetSelection> {
    let arch_key = architecture
        .release_artifact_token()
        .map(|arch_token| format!("{}-{}", platform_token(platform), arch_token));

    // A FRABBIT running from a `.app` must update from a bundle: dropping a
    // bare binary inside the running bundle would leave Info.plist, the
    // localization layout and the code signature describing the old version.
    // No silent fallback to the executable asset for that reason — a manifest
    // without a bundle entry means "not updatable from here", and the caller
    // points the user at the releases page instead.
    if preferred_kind == SelfUpdateAssetKind::AppBundle {
        let key = arch_key.clone().ok_or_else(|| FrabbitError::RemoteData {
            url: manifest_url.to_string(),
            message: format!(
                "no manifest asset for {platform:?} on architecture {architecture:?}: \
                 architecture is not produced by the FRABBIT release pipeline."
            ),
        })?;
        let asset = manifest
            .assets
            .bundles
            .as_ref()
            .and_then(|bundles| bundles.get(&key))
            .ok_or_else(|| FrabbitError::RemoteData {
                url: manifest_url.to_string(),
                message: format!(
                    "manifest does not list a {key} application bundle; \
                     download the matching build from the GitHub releases page manually."
                ),
            })?;
        return Ok(SelfUpdateAssetSelection {
            platform,
            url: asset.url.clone(),
            sha256: asset.sha256.clone(),
            kind: SelfUpdateAssetKind::AppBundle,
        });
    }

    if let Some(platforms) = &manifest.assets.platforms {
        let key = arch_key.ok_or_else(|| FrabbitError::RemoteData {
            url: manifest_url.to_string(),
            message: format!(
                "no manifest asset for {platform:?} on architecture {architecture:?}: \
                 architecture is not produced by the FRABBIT release pipeline."
            ),
        })?;
        let asset = platforms
            .get(&key)
            .ok_or_else(|| FrabbitError::RemoteData {
                url: manifest_url.to_string(),
                message: format!(
                    "manifest does not list a {key} asset; \
                 download the matching build from the GitHub releases page manually."
                ),
            })?;
        return Ok(SelfUpdateAssetSelection {
            platform,
            url: asset.url.clone(),
            sha256: asset.sha256.clone(),
            kind: SelfUpdateAssetKind::Executable,
        });
    }

    let asset = match platform {
        Platform::Windows => manifest.assets.windows.as_ref(),
        Platform::MacOs => manifest.assets.macos.as_ref(),
    }
    .ok_or_else(|| FrabbitError::RemoteData {
        url: manifest_url.to_string(),
        message: format!("missing asset entry for platform {platform:?}"),
    })?;

    if let (Some(expected), Some(actual)) = (
        architecture.release_artifact_token(),
        arch_token_from_asset_url(&asset.url),
    ) && expected != actual
    {
        return Err(FrabbitError::RemoteData {
            url: manifest_url.to_string(),
            message: format!(
                "self-update asset is built for {actual} but FRABBIT is running on {expected}; \
                 refusing to overwrite this binary with one for the wrong architecture. \
                 Download the matching build from the GitHub releases page manually."
            ),
        });
    }

    Ok(SelfUpdateAssetSelection {
        platform,
        url: asset.url.clone(),
        sha256: asset.sha256.clone(),
        kind: SelfUpdateAssetKind::Executable,
    })
}

fn platform_token(platform: Platform) -> &'static str {
    match platform {
        Platform::Windows => "windows",
        Platform::MacOs => "macos",
    }
}

fn validate_platform_key(key: &str, manifest_url: &str) -> Result<()> {
    let (os, arch) = key
        .split_once('-')
        .ok_or_else(|| FrabbitError::RemoteData {
            url: manifest_url.to_string(),
            message: format!("manifest platforms key '{key}' must be '<os>-<arch>'"),
        })?;
    let os_ok = matches!(os, "windows" | "macos");
    let arch_ok = matches!(arch, "x86_64" | "aarch64" | "i686" | "armv7");
    if !os_ok || !arch_ok {
        return Err(FrabbitError::RemoteData {
            url: manifest_url.to_string(),
            message: format!(
                "manifest platforms key '{key}' uses an unrecognised os or arch token"
            ),
        });
    }
    Ok(())
}

fn arch_token_from_asset_url(url: &str) -> Option<&str> {
    let basename = url.rsplit_once('/').map(|(_, name)| name).unwrap_or(url);
    let stem = basename.strip_suffix(".exe").unwrap_or(basename);
    let rest = stem.strip_prefix("frabbit-")?;
    let (_, arch) = rest.rsplit_once('-')?;
    match arch {
        "x86_64" | "aarch64" | "i686" | "armv7" => Some(arch),
        _ => None,
    }
}

/// Validate and convert one of the `<os>-<arch>`-keyed asset maps (`platforms`,
/// `bundles`). Both use the same key grammar, so both reject the same typos.
fn parse_keyed_assets(
    raw: Option<&BTreeMap<String, RawSelfUpdateAsset>>,
    manifest_url: &str,
) -> Result<Option<BTreeMap<String, SelfUpdateAsset>>> {
    raw.map(|entries| {
        entries
            .iter()
            .map(|(key, asset)| {
                validate_platform_key(key, manifest_url)?;
                let parsed = parse_asset(asset, manifest_url, key)?;
                Ok::<_, FrabbitError>((key.clone(), parsed))
            })
            .collect::<Result<BTreeMap<_, _>>>()
    })
    .transpose()
}

fn parse_asset(
    asset: &RawSelfUpdateAsset,
    manifest_url: &str,
    field: &str,
) -> Result<SelfUpdateAsset> {
    if !asset.url.starts_with("https://") {
        return Err(FrabbitError::RemoteData {
            url: manifest_url.to_string(),
            message: format!("{field} asset url must use https: {}", asset.url),
        });
    }
    if !is_valid_sha256(&asset.sha256) {
        return Err(FrabbitError::RemoteData {
            url: manifest_url.to_string(),
            message: format!("{field} asset sha256 must be 64 lowercase hexadecimal characters"),
        });
    }

    Ok(SelfUpdateAsset {
        url: asset.url.clone(),
        sha256: asset.sha256.clone(),
    })
}

fn parse_semantic_version(raw: &str, url: &str, field: &str) -> Result<Version> {
    semantic_version_from_str(raw, url, field)?;
    Version::parse(raw)
}

fn semantic_version_from_version(
    version: &Version,
    url: &str,
    field: &str,
) -> Result<SemanticVersion> {
    semantic_version_from_str(version.raw(), url, field)
}

fn semantic_version_from_str(raw: &str, url: &str, field: &str) -> Result<SemanticVersion> {
    let trimmed = raw.trim();
    let parts = trimmed.split('.').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(FrabbitError::RemoteData {
            url: url.to_string(),
            message: format!("{field} must use semantic versioning (major.minor.patch): {trimmed}"),
        });
    }

    let parse_part = |name: &str, value: &str| {
        value.parse::<u64>().map_err(|_| FrabbitError::RemoteData {
            url: url.to_string(),
            message: format!("{field} contains a non-numeric {name} segment: {trimmed}"),
        })
    };

    Ok(SemanticVersion {
        major: parse_part("major", parts[0])?,
        minor: parse_part("minor", parts[1])?,
        patch: parse_part("patch", parts[2])?,
    })
}

fn is_valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::{
        SelfUpdateAssetKind, arch_token_from_asset_url, current_frabbit_version,
        evaluate_self_update_report, parse_self_update_manifest,
    };
    use crate::model::{Architecture, Platform};
    use crate::version::Version;

    const MANIFEST_URL: &str = "https://example.test/frabbit-update-stable.json";

    #[test]
    fn parses_valid_self_update_manifest() {
        let manifest = parse_self_update_manifest(
            r#"{
              "version": "0.2.0",
              "channel": "stable",
              "published_at": "2026-04-25T00:00:00Z",
              "release_notes_url": "https://example.test/releases/v0.2.0",
              "minimum_supported_previous_version": "0.1.0",
              "assets": {
                "windows": {
                  "url": "https://example.test/FRABBIT-windows.zip",
                  "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                },
                "macos": {
                  "url": "https://example.test/FRABBIT-macos.zip",
                  "sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                }
              }
            }"#,
            MANIFEST_URL,
        )
        .unwrap();

        assert_eq!(manifest.version.raw(), "0.2.0");
        assert_eq!(manifest.channel, "stable");
        assert_eq!(
            manifest
                .minimum_supported_previous_version
                .as_ref()
                .unwrap()
                .raw(),
            "0.1.0"
        );
    }

    #[test]
    fn rejects_non_semantic_manifest_version() {
        let error = parse_self_update_manifest(
            r#"{
              "version": "0.2",
              "channel": "stable",
              "published_at": "2026-04-25T00:00:00Z",
              "assets": {
                "windows": {
                  "url": "https://example.test/FRABBIT-windows.zip",
                  "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                }
              }
            }"#,
            MANIFEST_URL,
        )
        .unwrap_err();

        assert!(error.to_string().contains("semantic versioning"));
    }

    #[test]
    fn rejects_non_https_asset_url() {
        let error = parse_self_update_manifest(
            r#"{
              "version": "0.2.0",
              "channel": "stable",
              "published_at": "2026-04-25T00:00:00Z",
              "assets": {
                "windows": {
                  "url": "http://example.test/FRABBIT-windows.zip",
                  "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                }
              }
            }"#,
            MANIFEST_URL,
        )
        .unwrap_err();

        assert!(error.to_string().contains("must use https"));
    }

    #[test]
    fn reports_update_available_for_newer_version() {
        let manifest = sample_manifest();

        let report = evaluate_self_update_report(
            Platform::Windows,
            Architecture::X64,
            MANIFEST_URL,
            Version::parse("0.1.0").unwrap(),
            &manifest,
            SelfUpdateAssetKind::Executable,
        )
        .unwrap();

        assert!(report.update_available);
        assert!(!report.requires_manual_transition);
        assert_eq!(report.asset.platform, Platform::Windows);
        assert!(report.asset.url.contains("FRABBIT-windows.zip"));
    }

    #[test]
    fn reports_manual_transition_requirement() {
        let manifest = sample_manifest();

        let report = evaluate_self_update_report(
            Platform::Windows,
            Architecture::X64,
            MANIFEST_URL,
            Version::parse("0.0.9").unwrap(),
            &manifest,
            SelfUpdateAssetKind::Executable,
        )
        .unwrap();

        assert!(report.update_available);
        assert!(report.requires_manual_transition);
    }

    #[test]
    fn arch_token_parser_extracts_known_archs() {
        assert_eq!(
            arch_token_from_asset_url("https://example.test/frabbit-0.2.0-windows-x86_64.exe"),
            Some("x86_64")
        );
        assert_eq!(
            arch_token_from_asset_url("https://example.test/frabbit-0.2.0-macos-aarch64"),
            Some("aarch64")
        );
        assert_eq!(
            arch_token_from_asset_url("https://example.test/FRABBIT-windows.zip"),
            None
        );
        assert_eq!(
            arch_token_from_asset_url("https://example.test/frabbit-0.2.0-linux-riscv64"),
            None
        );
    }

    #[test]
    fn refuses_self_update_when_asset_arch_mismatches_runtime() {
        let manifest = parse_self_update_manifest(
            r#"{
              "version": "0.2.0",
              "channel": "stable",
              "published_at": "2026-04-25T00:00:00Z",
              "assets": {
                "windows": {
                  "url": "https://example.test/frabbit-0.2.0-windows-x86_64.exe",
                  "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                }
              }
            }"#,
            MANIFEST_URL,
        )
        .unwrap();

        let error = evaluate_self_update_report(
            Platform::Windows,
            Architecture::Arm64,
            MANIFEST_URL,
            Version::parse("0.1.0").unwrap(),
            &manifest,
            SelfUpdateAssetKind::Executable,
        )
        .unwrap_err();

        let message = error.to_string();
        assert!(message.contains("x86_64"), "message was: {message}");
        assert!(message.contains("aarch64"), "message was: {message}");
    }

    #[test]
    fn allows_self_update_when_asset_arch_matches_runtime() {
        let manifest = parse_self_update_manifest(
            r#"{
              "version": "0.2.0",
              "channel": "stable",
              "published_at": "2026-04-25T00:00:00Z",
              "assets": {
                "macos": {
                  "url": "https://example.test/frabbit-0.2.0-macos-aarch64",
                  "sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                }
              }
            }"#,
            MANIFEST_URL,
        )
        .unwrap();

        let report = evaluate_self_update_report(
            Platform::MacOs,
            Architecture::Arm64,
            MANIFEST_URL,
            Version::parse("0.1.0").unwrap(),
            &manifest,
            SelfUpdateAssetKind::Executable,
        )
        .unwrap();

        assert!(report.update_available);
        assert!(report.asset.url.ends_with("frabbit-0.2.0-macos-aarch64"));
    }

    #[test]
    fn per_arch_platforms_table_is_authoritative_when_present() {
        let manifest = parse_self_update_manifest(
            r#"{
              "version": "0.2.0",
              "channel": "stable",
              "published_at": "2026-04-25T00:00:00Z",
              "assets": {
                "windows": {
                  "url": "https://example.test/frabbit-0.2.0-windows-x86_64.exe",
                  "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                },
                "macos": {
                  "url": "https://example.test/frabbit-0.2.0-macos-aarch64",
                  "sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                },
                "platforms": {
                  "windows-x86_64": {
                    "url": "https://example.test/frabbit-0.2.0-windows-x86_64.exe",
                    "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                  },
                  "windows-aarch64": {
                    "url": "https://example.test/frabbit-0.2.0-windows-aarch64.exe",
                    "sha256": "1111111111111111111111111111111111111111111111111111111111111111"
                  },
                  "macos-aarch64": {
                    "url": "https://example.test/frabbit-0.2.0-macos-aarch64",
                    "sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                  },
                  "macos-x86_64": {
                    "url": "https://example.test/frabbit-0.2.0-macos-x86_64",
                    "sha256": "2222222222222222222222222222222222222222222222222222222222222222"
                  }
                }
              }
            }"#,
            MANIFEST_URL,
        )
        .unwrap();

        let windows_arm = evaluate_self_update_report(
            Platform::Windows,
            Architecture::Arm64,
            MANIFEST_URL,
            Version::parse("0.1.0").unwrap(),
            &manifest,
            SelfUpdateAssetKind::Executable,
        )
        .unwrap();
        assert!(windows_arm.asset.url.ends_with("windows-aarch64.exe"));

        let macos_intel = evaluate_self_update_report(
            Platform::MacOs,
            Architecture::X64,
            MANIFEST_URL,
            Version::parse("0.1.0").unwrap(),
            &manifest,
            SelfUpdateAssetKind::Executable,
        )
        .unwrap();
        assert!(macos_intel.asset.url.ends_with("macos-x86_64"));
    }

    #[test]
    fn per_arch_platforms_table_errors_for_missing_arch() {
        let manifest = parse_self_update_manifest(
            r#"{
              "version": "0.2.0",
              "channel": "stable",
              "published_at": "2026-04-25T00:00:00Z",
              "assets": {
                "windows": {
                  "url": "https://example.test/frabbit-0.2.0-windows-x86_64.exe",
                  "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                },
                "platforms": {
                  "windows-x86_64": {
                    "url": "https://example.test/frabbit-0.2.0-windows-x86_64.exe",
                    "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                  }
                }
              }
            }"#,
            MANIFEST_URL,
        )
        .unwrap();

        let error = evaluate_self_update_report(
            Platform::Windows,
            Architecture::Arm64,
            MANIFEST_URL,
            Version::parse("0.1.0").unwrap(),
            &manifest,
            SelfUpdateAssetKind::Executable,
        )
        .unwrap_err();
        assert!(error.to_string().contains("windows-aarch64"));
    }

    /// A manifest carrying both flavours, as the release pipeline publishes.
    const MANIFEST_WITH_BUNDLES: &str = r#"{
      "version": "0.2.0",
      "channel": "stable",
      "published_at": "2026-04-25T00:00:00Z",
      "assets": {
        "windows": null,
        "macos": {
          "url": "https://example.test/frabbit-0.2.0-macos-universal",
          "sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
        },
        "platforms": {
          "macos-aarch64": {
            "url": "https://example.test/frabbit-0.2.0-macos-universal",
            "sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
          }
        },
        "bundles": {
          "macos-aarch64": {
            "url": "https://example.test/frabbit-0.2.0-macos-universal.app.zip",
            "sha256": "3333333333333333333333333333333333333333333333333333333333333333"
          }
        }
      }
    }"#;

    #[test]
    fn selects_the_app_bundle_when_running_from_one() {
        let manifest = parse_self_update_manifest(MANIFEST_WITH_BUNDLES, MANIFEST_URL).unwrap();

        let report = evaluate_self_update_report(
            Platform::MacOs,
            Architecture::Arm64,
            MANIFEST_URL,
            Version::parse("0.1.0").unwrap(),
            &manifest,
            SelfUpdateAssetKind::AppBundle,
        )
        .unwrap();

        assert_eq!(report.asset.kind, SelfUpdateAssetKind::AppBundle);
        assert!(report.asset.url.ends_with(".app.zip"));
    }

    #[test]
    fn a_bare_binary_still_gets_the_executable_asset_from_the_same_manifest() {
        let manifest = parse_self_update_manifest(MANIFEST_WITH_BUNDLES, MANIFEST_URL).unwrap();

        let report = evaluate_self_update_report(
            Platform::MacOs,
            Architecture::Arm64,
            MANIFEST_URL,
            Version::parse("0.1.0").unwrap(),
            &manifest,
            SelfUpdateAssetKind::Executable,
        )
        .unwrap();

        assert_eq!(report.asset.kind, SelfUpdateAssetKind::Executable);
        assert!(report.asset.url.ends_with("macos-universal"));
    }

    /// Never silently hand a bare binary to a FRABBIT running from a bundle:
    /// dropping it inside the `.app` would leave Info.plist and the code
    /// signature describing the previous version.
    #[test]
    fn refuses_to_fall_back_to_the_executable_when_a_bundle_is_required() {
        let manifest = parse_self_update_manifest(
            r#"{
              "version": "0.2.0",
              "channel": "stable",
              "published_at": "2026-04-25T00:00:00Z",
              "assets": {
                "windows": null,
                "macos": null,
                "platforms": {
                  "macos-aarch64": {
                    "url": "https://example.test/frabbit-0.2.0-macos-universal",
                    "sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                  }
                }
              }
            }"#,
            MANIFEST_URL,
        )
        .unwrap();

        let error = evaluate_self_update_report(
            Platform::MacOs,
            Architecture::Arm64,
            MANIFEST_URL,
            Version::parse("0.1.0").unwrap(),
            &manifest,
            SelfUpdateAssetKind::AppBundle,
        )
        .unwrap_err();
        let message = error.to_string();
        assert!(message.contains("macos-aarch64"), "message was: {message}");
        assert!(
            message.contains("application bundle"),
            "message was: {message}"
        );
    }

    #[test]
    fn rejects_manifest_with_unknown_bundles_key() {
        let error = parse_self_update_manifest(
            r#"{
              "version": "0.2.0",
              "channel": "stable",
              "published_at": "2026-04-25T00:00:00Z",
              "assets": {
                "windows": null,
                "macos": null,
                "bundles": {
                  "macos-aarch64-bundle": {
                    "url": "https://example.test/frabbit-0.2.0-macos.app.zip",
                    "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                  }
                }
              }
            }"#,
            MANIFEST_URL,
        )
        .unwrap_err();
        assert!(error.to_string().contains("macos-aarch64-bundle"));
    }

    #[test]
    fn rejects_manifest_with_unknown_platforms_key() {
        let error = parse_self_update_manifest(
            r#"{
              "version": "0.2.0",
              "channel": "stable",
              "published_at": "2026-04-25T00:00:00Z",
              "assets": {
                "platforms": {
                  "linux-x86_64": {
                    "url": "https://example.test/frabbit-0.2.0-linux-x86_64",
                    "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                  }
                }
              }
            }"#,
            MANIFEST_URL,
        )
        .unwrap_err();
        assert!(error.to_string().contains("unrecognised"));
    }

    #[test]
    fn current_build_version_is_semantic() {
        let version = current_frabbit_version().unwrap();
        assert_eq!(version.raw(), env!("CARGO_PKG_VERSION"));
    }

    fn sample_manifest() -> super::SelfUpdateManifest {
        parse_self_update_manifest(
            r#"{
              "version": "0.2.0",
              "channel": "stable",
              "published_at": "2026-04-25T00:00:00Z",
              "release_notes_url": "https://example.test/releases/v0.2.0",
              "minimum_supported_previous_version": "0.1.0",
              "assets": {
                "windows": {
                  "url": "https://example.test/FRABBIT-windows.zip",
                  "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                },
                "macos": {
                  "url": "https://example.test/FRABBIT-macos.zip",
                  "sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                }
              }
            }"#,
            MANIFEST_URL,
        )
        .unwrap()
    }
}
