use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, FrabbitError>;

#[derive(Debug, Error)]
pub enum FrabbitError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("JSON error at {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("SQLite error at {path}: {source}")]
    Sqlite {
        path: PathBuf,
        #[source]
        source: rusqlite::Error,
    },

    #[error("HTTP error for {url}: {source}")]
    Http {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("remote data error for {url}: {message}")]
    RemoteData { url: String, message: String },

    #[error("invalid artifact URL {url}: {message}")]
    InvalidArtifactUrl { url: String, message: String },

    #[error("hash mismatch for {path}: expected {expected}, got {actual}")]
    HashMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },

    #[error("no artifact found for {package_id} on {platform:?}/{architecture:?}")]
    NoArtifactFound {
        package_id: String,
        platform: crate::model::Platform,
        architecture: crate::model::Architecture,
    },

    #[error("artifact kind {kind:?} for {package_id} is not supported by this installer step")]
    UnsupportedArtifactKind {
        package_id: String,
        kind: crate::artifact::ArtifactKind,
    },

    #[error("archive {archive} for {package_id} did not contain a {package_id} extension binary")]
    ArchiveMissingExtensionBinary {
        archive: PathBuf,
        package_id: String,
    },

    #[error("archive {archive} could not be read: {message}")]
    ArchiveRead { archive: PathBuf, message: String },

    #[error("archive {archive} did not contain the expected OSARA installer assets")]
    OsaraArchiveMissingAssets { archive: PathBuf },

    #[error("disk image {image} for {package_id} did not contain a {package_id} extension binary")]
    DiskImageMissingExtensionBinary { image: PathBuf, package_id: String },

    #[error("disk image {image} did not contain the expected app bundle {bundle}")]
    DiskImageMissingAppBundle { image: PathBuf, bundle: String },

    #[error("disk image {image} could not be mounted: {message}")]
    DiskImageMount { image: PathBuf, message: String },

    #[error("a package installation is already in progress (lock {lock_path}, PID {pid})")]
    PackageInstallInProgress { lock_path: PathBuf, pid: u32 },

    #[error("preflight failed: {message}")]
    PreflightFailed { message: String },

    #[error("invalid planned execution: {message}")]
    InvalidPlannedExecution { message: String },

    #[error("process failed for {program} with exit code {exit_code:?}")]
    ProcessFailed {
        program: String,
        exit_code: Option<i32>,
    },

    /// Raised on Windows (UAC consent declined) and on macOS (the admin
    /// authorization dialog dismissed), so the wording stays platform
    /// neutral. The wizard replaces it with a localized, platform-specific
    /// sentence; this text is what lands in reports and CLI output.
    #[error(
        "the administrator approval prompt for {program} was cancelled or declined; nothing was installed — re-run the installation and approve the prompt to continue"
    )]
    UserCancelledElevation { program: String },

    /// The macOS `installer` command ran but exited non-zero. Distinct from
    /// [`FrabbitError::ProcessFailed`] so the wizard can name the disk image
    /// and point at the report instead of surfacing `/usr/sbin/installer`.
    #[error("the macOS installer for {image} failed with exit code {exit_code:?}")]
    PkgInstallerFailed {
        image: PathBuf,
        exit_code: Option<i32>,
    },

    /// An app bundle could not be written to its install destination —
    /// typically `/Applications` not being writable. Distinct from
    /// [`FrabbitError::Io`] so the wizard can talk about permissions on a
    /// named folder rather than reporting a raw OS error number.
    #[error("the app bundle could not be installed into {destination}: {message}")]
    AppBundleInstallFailed {
        destination: PathBuf,
        message: String,
    },

    #[error("post-install verification failed; missing paths: {missing_paths:?}")]
    PostInstallVerificationFailed { missing_paths: Vec<PathBuf> },

    #[error("invalid version string: {0}")]
    InvalidVersion(String),

    #[error("localization error: {message}")]
    Localization {
        path: Option<PathBuf>,
        message: String,
    },

    #[error("unsupported platform")]
    UnsupportedPlatform,
}

pub trait IoPathContext<T> {
    fn with_path(self, path: impl Into<PathBuf>) -> Result<T>;
}

impl<T> IoPathContext<T> for std::io::Result<T> {
    fn with_path(self, path: impl Into<PathBuf>) -> Result<T> {
        let path = path.into();
        self.map_err(|source| FrabbitError::Io { path, source })
    }
}

pub trait JsonPathContext<T> {
    fn with_json_path(self, path: impl Into<PathBuf>) -> Result<T>;
}

impl<T> JsonPathContext<T> for serde_json::Result<T> {
    fn with_json_path(self, path: impl Into<PathBuf>) -> Result<T> {
        let path = path.into();
        self.map_err(|source| FrabbitError::Json { path, source })
    }
}

pub trait SqlitePathContext<T> {
    fn with_sqlite_path(self, path: impl Into<PathBuf>) -> Result<T>;
}

impl<T> SqlitePathContext<T> for rusqlite::Result<T> {
    fn with_sqlite_path(self, path: impl Into<PathBuf>) -> Result<T> {
        let path = path.into();
        self.map_err(|source| FrabbitError::Sqlite { path, source })
    }
}
