//! Thin wrapper around `frabbit_platform::signature` so callers in `frabbit-core`
//! get a `Result<_, FrabbitError>` instead of `std::io::Result`. The verdict
//! type, codesign/signtool dispatch, and tampering classifier all live in
//! `frabbit-platform` — see that crate's docs for the verdict semantics.

use std::path::Path;

pub use frabbit_platform::SignatureVerdict;

use crate::Result;
use crate::error::FrabbitError;

pub fn verify_executable_signature(path: &Path) -> Result<SignatureVerdict> {
    frabbit_platform::verify_executable_signature(path).map_err(|source| FrabbitError::Io {
        path: path.to_path_buf(),
        source,
    })
}
