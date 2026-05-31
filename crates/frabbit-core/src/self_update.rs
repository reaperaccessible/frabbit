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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfUpdateAsset {
    pub url: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfUpdateAssetSelection {
    pub platform: Platform,
    pub url: String,
    pub sha256: String,
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
    let platforms = raw
        .assets
        .platforms
        .as_ref()
        .map(|raw_platforms| {
            raw_platforms
                .iter()
                .map(|(key, asset)| {
                    validate_platform_key(key, manifest_url)?;
                    let parsed = parse_asset(asset, manifest_url, key)?;
                    Ok::<_, FrabbitError>((key.clone(), parsed))
                })
                .collect::<Result<BTreeMap<_, _>>>()
        })
        .transpose()?;
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
    let manifest = fetch_self_update_manifest(manifest_url)?;
    evaluate_self_update_report(
        platform,
        Architecture::current(),
        manifest_url,
        current_frabbit_version()?,
        &manifest,
    )
}

fn evaluate_self_update_report(
    platform: Platform,
    architecture: Architecture,
    manifest_url: &str,
    current_version: Version,
    manifest: &SelfUpdateManifest,
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
        asset: select_asset_for_platform(platform, architecture, manifest, manifest_url)?,
    })
}

fn select_asset_for_platform(
    platform: Platform,
    architecture: Architecture,
    manifest: &SelfUpdateManifest,
    manifest_url: &str,
) -> Result<SelfUpdateAssetSelection> {
    if let Some(platforms) = &manifest.assets.platforms {
        let arch_token =
            architecture
                .release_artifact_token()
                .ok_or_else(|| FrabbitError::RemoteData {
                    url: manifest_url.to_string(),
                    message: format!(
                        "no manifest asset for {platform:?} on architecture {architecture:?}: \
                     architecture is not produced by the FRABBIT release pipeline."
                    ),
                })?;
        let key = format!("{}-{}", platform_token(platform), arch_token);
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
        arch_token_from_asset_url, current_frabbit_version, evaluate_self_update_report,
        parse_self_update_manifest,
    };
    use crate::FrabbitError;
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
        )
        .unwrap();
        assert!(windows_arm.asset.url.ends_with("windows-aarch64.exe"));

        let macos_intel = evaluate_self_update_report(
            Platform::MacOs,
            Architecture::X64,
            MANIFEST_URL,
            Version::parse("0.1.0").unwrap(),
            &manifest,
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
        )
        .unwrap_err();
        assert!(error.to_string().contains("windows-aarch64"));
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
