use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::{IoPathContext, Result, SqlitePathContext};
use crate::version::Version;

pub const REAPACK_REGISTRY_RELATIVE_PATH: &str = "ReaPack/registry.db";
pub const REAPACK_INI_RELATIVE_PATH: &str = "reapack.ini";

/// The `[general] version=` marker ReaPack uses to decide whether to run
/// its first-time migration. ReaPack's `migrate()` only ever *seeds its
/// default repositories* (ReaTeam Extensions/Scripts, MPL, etc.) at each
/// version bump; if `reapack.ini` already declares a version equal to (or
/// greater than) ReaPack's current one, that migration is a no-op and the
/// default repos are never added. Pre-writing this value before REAPER
/// first launches is therefore how FRABBIT keeps ReaPack from pulling in
/// thousands of scripts the user never asked for. 4 matches the version
/// current ReaPack writes.
pub const REAPACK_CURATED_VERSION: u32 = 4;

/// Outcome of [`upsert_remote`]: tells callers whether the upsert was a
/// no-op (the URL was already configured), whether we appended into an
/// existing config, or whether we created a fresh `reapack.ini` (which
/// happens when ReaPack hasn't run yet — the user's first REAPER launch
/// will let ReaPack's own first-time migration add its default ReaTeam
/// repos alongside the remote we just wrote).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteUpsertOutcome {
    /// A remote with this URL was already configured under the same name;
    /// nothing changed.
    AlreadyPresent,
    /// Appended a new remote into the existing `reapack.ini`.
    Added,
    /// `reapack.ini` did not exist; created it with our remote as the
    /// only entry.
    CreatedFile,
    /// The URL was already configured but under a *different* name (an
    /// older FRABBIT wrote e.g. "ReaperAccessible FR"). ReaPack uses the
    /// remote name as the on-disk scripts folder, so the name was
    /// corrected in place to match the repository's canonical name.
    Renamed,
}

/// Add a remote repository to ReaPack's `reapack.ini` config file.
/// Idempotent on URL: re-running with the same URL is a no-op rather
/// than appending a duplicate. Field encoding mirrors what ReaPack
/// itself writes — `name|url|enabled|autoinstall`, where `enabled` is
/// `0`/`1` and `autoinstall` is `0` (false), `1` (true), or `2` (use
/// the global default; what ReaPack writes for repos that don't
/// override the global setting).
pub fn upsert_remote(resource_path: &Path, name: &str, url: &str) -> Result<RemoteUpsertOutcome> {
    let ini_path = resource_path.join(REAPACK_INI_RELATIVE_PATH);
    let original = if ini_path.is_file() {
        fs::read_to_string(&ini_path).with_path(&ini_path)?
    } else {
        String::new()
    };
    let creating_new = original.is_empty();
    // Preserve the file's existing line ending — Windows users typically
    // have `\r\n` here, and overwriting to `\n` would visually scramble the
    // file in tools that don't normalise.
    let newline = if original.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };

    // If the URL is already configured, it's either under the correct name
    // (nothing to do) or under a stale one. ReaPack derives the on-disk
    // scripts folder from the remote name, so a stale name (an older FRABBIT
    // wrote "ReaperAccessible FR" before the name was corrected) must be
    // fixed in place — otherwise re-running FRABBIT with the right name is a
    // silent no-op and the folder stays wrong.
    if !creating_new {
        if let Some(existing_name) = remote_name_for_url(&original, url) {
            if existing_name == name {
                return Ok(RemoteUpsertOutcome::AlreadyPresent);
            }
            let renamed = rewrite_remote_name(&original, url, name, newline);
            fs::create_dir_all(resource_path).with_path(resource_path)?;
            fs::write(&ini_path, renamed).with_path(&ini_path)?;
            return Ok(RemoteUpsertOutcome::Renamed);
        }
    }

    let new_text = if creating_new {
        format!(
            "[remotes]{nl}size=1{nl}remote0={name}|{url}|1|2{nl}",
            nl = newline,
            name = name,
            url = url,
        )
    } else {
        rewrite_with_appended_remote(&original, name, url, newline)
    };

    fs::create_dir_all(resource_path).with_path(resource_path)?;
    fs::write(&ini_path, new_text).with_path(&ini_path)?;

    Ok(if creating_new {
        RemoteUpsertOutcome::CreatedFile
    } else {
        RemoteUpsertOutcome::Added
    })
}

/// Outcome of [`apply_curated_reapack_defaults`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CuratedDefaultsOutcome {
    /// `reapack.ini` already declared a `[general] version`, meaning
    /// ReaPack has run and the file reflects the user's own choices.
    /// FRABBIT left it completely untouched — no repository is removed
    /// and no setting is overridden.
    LeftExisting,
    /// A FRABBIT-created `reapack.ini` (e.g. one that only had the
    /// `[remotes]` section) was updated in place to add the curated
    /// settings.
    Updated,
    /// `reapack.ini` did not exist; created it with the curated settings.
    CreatedFile,
}

/// Write FRABBIT's curated ReaPack defaults into a *fresh* `reapack.ini`
/// so a first-time ReaPack install doesn't force unwanted repositories on
/// the user:
///
/// * `[general] version` is set to [`REAPACK_CURATED_VERSION`], which
///   stops ReaPack's first-launch migration from seeding its default
///   ReaTeam / MPL repositories.
/// * `[install] autoinstall` is set to `0`, so even the one repository
///   ReaPack always re-adds for its own updates (`reapack.com`, which
///   cannot be permanently removed) never mass-installs packages when the
///   user synchronizes.
///
/// **Existing configs are never touched.** If `reapack.ini` already
/// declares a `[general] version`, ReaPack has already run and the file
/// reflects the user's own decisions — including any default repositories
/// they deliberately kept. In that case this returns
/// [`CuratedDefaultsOutcome::LeftExisting`] without changing a single
/// byte: FRABBIT removes no repository and overrides no setting. We only
/// ever pre-empt the fresh case, which is the one that seeds thousands of
/// scripts. A file FRABBIT itself created this run (e.g. one holding only
/// the `[remotes]` entry written by [`upsert_remote`]) has no version yet,
/// so it is treated as fresh and the curated settings are added while its
/// `[remotes]` section is preserved.
pub fn apply_curated_reapack_defaults(resource_path: &Path) -> Result<CuratedDefaultsOutcome> {
    let ini_path = resource_path.join(REAPACK_INI_RELATIVE_PATH);
    let original = if ini_path.is_file() {
        fs::read_to_string(&ini_path).with_path(&ini_path)?
    } else {
        String::new()
    };
    let creating_new = original.is_empty();
    let newline = if original.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };

    let mut lines: Vec<String> = if creating_new {
        Vec::new()
    } else {
        original.lines().map(|s| s.to_string()).collect()
    };

    // A declared version means ReaPack owns this file already — hands off.
    if ini_value(&lines, "general", "version").is_some() {
        return Ok(CuratedDefaultsOutcome::LeftExisting);
    }

    set_ini_value(
        &mut lines,
        "general",
        "version",
        &REAPACK_CURATED_VERSION.to_string(),
    );
    set_ini_value(&mut lines, "install", "autoinstall", "0");

    let mut joined = lines.join(newline);
    if !joined.ends_with(newline) {
        joined.push_str(newline);
    }
    fs::create_dir_all(resource_path).with_path(resource_path)?;
    fs::write(&ini_path, joined).with_path(&ini_path)?;

    Ok(if creating_new {
        CuratedDefaultsOutcome::CreatedFile
    } else {
        CuratedDefaultsOutcome::Updated
    })
}

/// Read `key=value` under `[section]` from already-split INI lines.
/// Returns the first match's raw (un-trimmed) value.
fn ini_value(lines: &[String], section: &str, key: &str) -> Option<String> {
    let mut in_target = false;
    for line in lines {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('[')
            && let Some(name) = rest.strip_suffix(']')
        {
            in_target = name == section;
            continue;
        }
        if in_target
            && let Some((k, v)) = trimmed.split_once('=')
            && k.trim() == key
        {
            return Some(v.to_string());
        }
    }
    None
}

/// Set `key=value` under `[section]`, creating the key at the end of the
/// section if absent, or appending a fresh `[section]` at end of file if
/// the section itself is missing. Every other line keeps its position.
fn set_ini_value(lines: &mut Vec<String>, section: &str, key: &str, value: &str) {
    let new_line = format!("{key}={value}");
    let mut in_target = false;
    let mut section_start: Option<usize> = None;
    let mut section_end: Option<usize> = None;
    let mut key_idx: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('[')
            && let Some(name) = rest.strip_suffix(']')
        {
            if in_target && section_end.is_none() {
                section_end = Some(i);
            }
            in_target = name == section;
            if in_target && section_start.is_none() {
                section_start = Some(i);
            }
            continue;
        }
        if in_target
            && key_idx.is_none()
            && let Some((k, _)) = trimmed.split_once('=')
            && k.trim() == key
        {
            key_idx = Some(i);
        }
    }
    if in_target && section_end.is_none() {
        section_end = Some(lines.len());
    }

    match (section_start, key_idx) {
        (Some(_), Some(idx)) => lines[idx] = new_line,
        (Some(_), None) => {
            let end = section_end.unwrap_or(lines.len());
            lines.insert(end, new_line);
        }
        (None, _) => {
            lines.push(format!("[{section}]"));
            lines.push(new_line);
        }
    }
}

/// `true` if `<resource_path>/reapack.ini` exists and lists `url` under
/// its `[remotes]` section. Used by the wizard / CLI to suppress the
/// "configure REAPER Accessibility ReaPack remote" step when the remote
/// is already wired up — there is nothing for us to do, and re-offering
/// it as actionable is misleading. A missing `reapack.ini` reads as
/// "not configured".
pub fn is_remote_configured(resource_path: &Path, url: &str) -> Result<bool> {
    let ini_path = resource_path.join(REAPACK_INI_RELATIVE_PATH);
    if !ini_path.is_file() {
        return Ok(false);
    }
    let text = fs::read_to_string(&ini_path).with_path(&ini_path)?;
    Ok(url_is_already_present(&text, url))
}

/// `true` when the `[remotes]` section already contains an entry whose
/// URL field matches `url` (the second pipe-delimited field of any
/// `remote<N>=...` line).
fn url_is_already_present(text: &str, url: &str) -> bool {
    let mut current_section: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('[')
            && let Some(name) = rest.strip_suffix(']')
        {
            current_section = Some(name.to_string());
            continue;
        }
        if current_section.as_deref() != Some("remotes") {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if !key.trim().starts_with("remote") {
            continue;
        }
        let mut parts = value.trim().splitn(4, '|');
        let _name = parts.next();
        if let Some(existing_url) = parts.next()
            && existing_url == url
        {
            return true;
        }
    }
    false
}

/// The name field of the `[remotes]` entry whose URL field matches `url`,
/// or `None` when no entry has that URL. Used to detect a stale name that
/// needs correcting.
fn remote_name_for_url(text: &str, url: &str) -> Option<String> {
    let mut current_section: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('[')
            && let Some(name) = rest.strip_suffix(']')
        {
            current_section = Some(name.to_string());
            continue;
        }
        if current_section.as_deref() != Some("remotes") {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if !key.trim().starts_with("remote") {
            continue;
        }
        let mut parts = value.trim().splitn(4, '|');
        let name = parts.next();
        if parts.next() == Some(url) {
            return name.map(|n| n.to_string());
        }
    }
    None
}

/// Rewrite the `[remotes]` entry whose URL matches `url` so its name field
/// becomes `new_name`, leaving every other field (URL, enabled, autoinstall)
/// and every other line untouched.
fn rewrite_remote_name(original: &str, url: &str, new_name: &str, newline: &str) -> String {
    let mut out: Vec<String> = Vec::with_capacity(original.lines().count() + 1);
    let mut in_remotes = false;
    for line in original.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('[')
            && let Some(section) = rest.strip_suffix(']')
        {
            in_remotes = section == "remotes";
            out.push(line.to_string());
            continue;
        }
        if in_remotes
            && let Some((key, value)) = trimmed.split_once('=')
            && key.trim().starts_with("remote")
        {
            let fields: Vec<&str> = value.trim().splitn(4, '|').collect();
            // fields = [name, url, enabled, autoinstall]
            if fields.len() >= 2 && fields[1] == url {
                let tail = fields[1..].join("|");
                out.push(format!("{}={}|{}", key.trim(), new_name, tail));
                continue;
            }
        }
        out.push(line.to_string());
    }
    let mut joined = out.join(newline);
    if !joined.ends_with(newline) {
        joined.push_str(newline);
    }
    joined
}

/// Append a `remote<size>=...` entry into the existing `[remotes]`
/// section and bump `size=`. If the section doesn't exist yet, append
/// it at the end of the file as a fresh section with our entry as
/// `remote0`.
fn rewrite_with_appended_remote(original: &str, name: &str, url: &str, newline: &str) -> String {
    let mut lines: Vec<String> = original.lines().map(|s| s.to_string()).collect();

    // First pass: locate the `[remotes]` section and pull out its size +
    // the highest existing remote index.
    let mut current_section: Option<String> = None;
    let mut section_start: Option<usize> = None;
    let mut section_end: Option<usize> = None;
    let mut size_line_idx: Option<usize> = None;
    let mut current_size: u32 = 0;
    let mut max_remote_idx: Option<u32> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('[')
            && let Some(sec_name) = rest.strip_suffix(']')
        {
            if current_section.as_deref() == Some("remotes") && section_end.is_none() {
                section_end = Some(i);
            }
            current_section = Some(sec_name.to_string());
            if sec_name == "remotes" && section_start.is_none() {
                section_start = Some(i);
            }
            continue;
        }
        if current_section.as_deref() != Some("remotes") {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key == "size" {
            current_size = value.trim().parse().unwrap_or(0);
            size_line_idx = Some(i);
        } else if let Some(idx_str) = key.strip_prefix("remote")
            && let Ok(idx) = idx_str.parse::<u32>()
        {
            max_remote_idx = Some(max_remote_idx.map_or(idx, |m| m.max(idx)));
        }
    }
    if current_section.as_deref() == Some("remotes") && section_end.is_none() {
        section_end = Some(lines.len());
    }

    // Pick the next index. ReaPack treats `size` as authoritative for the
    // count, so we honour that — but we also fall back to "max existing
    // index + 1" if `size` is missing or out of sync.
    let next_index = current_size.max(max_remote_idx.map_or(0, |m| m + 1));
    let new_size = next_index + 1;
    let new_remote_line = format!("remote{}={}|{}|1|2", next_index, name, url);
    let new_size_line = format!("size={}", new_size);

    if let (Some(start), Some(end)) = (section_start, section_end) {
        // Insert the new remote line at the bottom of the existing
        // `[remotes]` section. Update or insert `size=`.
        if let Some(idx) = size_line_idx {
            lines[idx] = new_size_line;
            lines.insert(end, new_remote_line);
        } else {
            lines.insert(start + 1, new_size_line);
            // The size insertion shifts the section_end by 1.
            lines.insert(end + 1, new_remote_line);
        }
    } else {
        // No `[remotes]` section yet — append a fresh one at the end.
        if !lines.is_empty() {
            let last_is_blank = lines.last().is_some_and(|line| line.trim().is_empty());
            if !last_is_blank {
                lines.push(String::new());
            }
        }
        lines.push("[remotes]".to_string());
        lines.push("size=1".to_string());
        lines.push(format!("remote0={}|{}|1|2", name, url));
    }

    let mut joined = lines.join(newline);
    if !original.ends_with(newline) || joined.is_empty() {
        // Match the source file's trailing-newline convention.
    }
    if !joined.ends_with(newline) {
        joined.push_str(newline);
    }
    joined
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReapackEntry {
    pub remote: String,
    pub category: String,
    pub package: String,
    pub version: Version,
}

/// Extract every `SCR` line from the textual content of a `reaper-kb.ini`.
/// ReaPack registers every installed ReaScript action exclusively via
/// these lines (it never writes `KEY` or `ACT` records itself; REAPER
/// produces the line in response to ReaPack's `AddRemoveReaScript`
/// host-API call). Returning them as opaque strings lets callers
/// re-append them verbatim to a freshly written key map — preserving the
/// `_RS<hex>` action command IDs, which REAPER derives deterministically
/// from each script's path, so any user `KEY` binding that targets one
/// of those IDs keeps working.
pub fn extract_scr_lines(reaper_kb_ini_text: &str) -> Vec<String> {
    reaper_kb_ini_text
        .lines()
        .filter(|line| line.starts_with("SCR ") || line.starts_with("SCR\t"))
        .map(|line| line.to_string())
        .collect()
}

pub fn registry_path(resource_path: &Path) -> PathBuf {
    resource_path.join(REAPACK_REGISTRY_RELATIVE_PATH)
}

pub fn package_owner_for_file(
    resource_path: &Path,
    absolute_file: &Path,
) -> Result<Option<ReapackEntry>> {
    let db_path = registry_path(resource_path);
    if !db_path.is_file() {
        return Ok(None);
    }

    let Some(relative_path) = relative_reapack_path(resource_path, absolute_file) else {
        return Ok(None);
    };

    query_owner(&db_path, &relative_path)
}

pub fn list_entries(resource_path: &Path) -> Result<Vec<ReapackEntry>> {
    let db_path = registry_path(resource_path);
    if !db_path.is_file() {
        return Ok(Vec::new());
    }

    let connection = open_read_only(&db_path)?;
    let mut statement = connection
        .prepare("SELECT remote, category, package, version FROM entries ORDER BY remote, category, package")
        .with_sqlite_path(&db_path)?;

    let rows = statement
        .query_map([], |row| {
            let version_text: String = row.get(3)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                version_text,
            ))
        })
        .with_sqlite_path(&db_path)?;

    let mut entries = Vec::new();
    for row in rows {
        let (remote, category, package, version_text) = row.with_sqlite_path(&db_path)?;
        let Ok(version) = Version::parse(&version_text) else {
            continue;
        };
        entries.push(ReapackEntry {
            remote,
            category,
            package,
            version,
        });
    }

    Ok(entries)
}

fn query_owner(db_path: &Path, relative_path: &str) -> Result<Option<ReapackEntry>> {
    let connection = open_read_only(db_path)?;
    let sql = "SELECT e.remote, e.category, e.package, e.version \
               FROM entries e JOIN files f ON f.entry = e.id \
               WHERE f.path = ?1 LIMIT 1";
    let mut statement = connection.prepare(sql).with_sqlite_path(db_path)?;

    let entry = statement
        .query_row([relative_path], |row| {
            let version_text: String = row.get(3)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                version_text,
            ))
        })
        .optional()
        .with_sqlite_path(db_path)?;

    let Some((remote, category, package, version_text)) = entry else {
        return Ok(None);
    };
    let Ok(version) = Version::parse(&version_text) else {
        return Ok(None);
    };

    Ok(Some(ReapackEntry {
        remote,
        category,
        package,
        version,
    }))
}

fn open_read_only(db_path: &Path) -> Result<Connection> {
    Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY).with_sqlite_path(db_path)
}

fn relative_reapack_path(resource_path: &Path, absolute_file: &Path) -> Option<String> {
    let canonical_resource = fs::canonicalize(resource_path)
        .with_path(resource_path)
        .ok()?;
    let canonical_file = fs::canonicalize(absolute_file)
        .with_path(absolute_file)
        .ok()?;
    let relative = canonical_file.strip_prefix(canonical_resource).ok()?;
    Some(relative.to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rusqlite::Connection;
    use tempfile::tempdir;

    use super::{
        CuratedDefaultsOutcome, REAPACK_INI_RELATIVE_PATH, RemoteUpsertOutcome,
        apply_curated_reapack_defaults, extract_scr_lines, is_remote_configured, list_entries,
        package_owner_for_file, upsert_remote,
    };

    const TEST_REPO_NAME: &str = "REAPER Accessibility";
    const TEST_REPO_URL: &str = "https://github.com/Timtam/reapack/raw/master/index.xml";

    #[test]
    fn upsert_creates_reapack_ini_when_missing() {
        let dir = tempdir().unwrap();
        let outcome = upsert_remote(dir.path(), TEST_REPO_NAME, TEST_REPO_URL).unwrap();
        assert_eq!(outcome, RemoteUpsertOutcome::CreatedFile);

        let ini = fs::read_to_string(dir.path().join(REAPACK_INI_RELATIVE_PATH)).unwrap();
        assert!(ini.contains("[remotes]"));
        assert!(ini.contains("size=1"));
        assert!(ini.contains(&format!("remote0={}|{}|1|2", TEST_REPO_NAME, TEST_REPO_URL)));
    }

    #[test]
    fn upsert_is_idempotent_on_url() {
        let dir = tempdir().unwrap();
        let first = upsert_remote(dir.path(), TEST_REPO_NAME, TEST_REPO_URL).unwrap();
        assert_eq!(first, RemoteUpsertOutcome::CreatedFile);
        let second = upsert_remote(dir.path(), TEST_REPO_NAME, TEST_REPO_URL).unwrap();
        assert_eq!(second, RemoteUpsertOutcome::AlreadyPresent);

        // The single entry should still be the only one.
        let ini = fs::read_to_string(dir.path().join(REAPACK_INI_RELATIVE_PATH)).unwrap();
        assert_eq!(ini.matches(TEST_REPO_URL).count(), 1);
        assert!(ini.contains("size=1"));
    }

    #[test]
    fn upsert_corrects_a_stale_remote_name_for_the_same_url() {
        // Older FRABBIT wrote the ReaperAccessible remote under the wrong
        // name; ReaPack made a `Scripts/ReaperAccessible FR/` folder from it.
        // Re-running with the canonical name must FIX the name in place, not
        // no-op on "URL already present".
        let dir = tempdir().unwrap();
        let ini_path = dir.path().join(REAPACK_INI_RELATIVE_PATH);
        fs::write(
            &ini_path,
            "[remotes]\nsize=1\nremote0=ReaperAccessible FR|https://github.com/reaperaccessible/rap_fr/raw/main/index.xml|1|2\n",
        )
        .unwrap();

        let outcome = upsert_remote(
            dir.path(),
            "ReaperAccessible scripts",
            "https://github.com/reaperaccessible/rap_fr/raw/main/index.xml",
        )
        .unwrap();
        assert_eq!(outcome, RemoteUpsertOutcome::Renamed);

        let ini = fs::read_to_string(&ini_path).unwrap();
        // Name corrected, URL and the rest of the line preserved, no duplicate.
        assert!(ini.contains(
            "remote0=ReaperAccessible scripts|https://github.com/reaperaccessible/rap_fr/raw/main/index.xml|1|2"
        ));
        assert!(!ini.contains("ReaperAccessible FR"));
        assert!(ini.contains("size=1"));
        assert_eq!(
            ini.matches("rap_fr/raw/main/index.xml").count(),
            1,
            "must not duplicate the remote"
        );
    }

    #[test]
    fn upsert_is_a_no_op_when_name_and_url_both_already_match() {
        let dir = tempdir().unwrap();
        let ini_path = dir.path().join(REAPACK_INI_RELATIVE_PATH);
        fs::write(
            &ini_path,
            format!("[remotes]\nsize=1\nremote0={TEST_REPO_NAME}|{TEST_REPO_URL}|1|2\n"),
        )
        .unwrap();
        let outcome = upsert_remote(dir.path(), TEST_REPO_NAME, TEST_REPO_URL).unwrap();
        assert_eq!(outcome, RemoteUpsertOutcome::AlreadyPresent);
    }

    #[test]
    fn upsert_appends_into_existing_remotes_section() {
        let dir = tempdir().unwrap();
        let ini_path = dir.path().join(REAPACK_INI_RELATIVE_PATH);
        // Mimic an existing reapack.ini with one prior entry.
        fs::write(
            &ini_path,
            "[reapack]\n\
             version=4\n\
             \n\
             [remotes]\n\
             size=1\n\
             remote0=ReaTeam Extensions|https://github.com/ReaTeam/Extensions/raw/master/index.xml|1|2\n",
        )
        .unwrap();

        let outcome = upsert_remote(dir.path(), TEST_REPO_NAME, TEST_REPO_URL).unwrap();
        assert_eq!(outcome, RemoteUpsertOutcome::Added);

        let ini = fs::read_to_string(&ini_path).unwrap();
        // Original remote preserved.
        assert!(ini.contains("remote0=ReaTeam Extensions|"));
        // Our remote appended at the next index.
        assert!(ini.contains(&format!("remote1={}|{}|1|2", TEST_REPO_NAME, TEST_REPO_URL)));
        // Size bumped.
        assert!(ini.contains("size=2"));
        // The unrelated [reapack] section is untouched.
        assert!(ini.contains("version=4"));
    }

    #[test]
    fn upsert_appends_at_size_when_indexes_have_gaps() {
        // ReaPack's `size` is the authoritative count; existing entries
        // could have gaps if a previous version removed remotes. Verify
        // we honour `size` when picking the next index.
        let dir = tempdir().unwrap();
        let ini_path = dir.path().join(REAPACK_INI_RELATIVE_PATH);
        fs::write(
            &ini_path,
            "[remotes]\n\
             size=10\n\
             remote0=ReaTeam Extensions|https://github.com/ReaTeam/Extensions/raw/master/index.xml|1|2\n\
             remote2=Old Repo|https://example.invalid/index.xml|1|2\n",
        )
        .unwrap();

        upsert_remote(dir.path(), TEST_REPO_NAME, TEST_REPO_URL).unwrap();

        let ini = fs::read_to_string(&ini_path).unwrap();
        assert!(ini.contains(&format!(
            "remote10={}|{}|1|2",
            TEST_REPO_NAME, TEST_REPO_URL
        )));
        assert!(ini.contains("size=11"));
    }

    #[test]
    fn upsert_creates_remotes_section_when_only_other_sections_exist() {
        let dir = tempdir().unwrap();
        let ini_path = dir.path().join(REAPACK_INI_RELATIVE_PATH);
        fs::write(&ini_path, "[reapack]\nversion=4\nlast_browse=\n").unwrap();

        let outcome = upsert_remote(dir.path(), TEST_REPO_NAME, TEST_REPO_URL).unwrap();
        assert_eq!(outcome, RemoteUpsertOutcome::Added);

        let ini = fs::read_to_string(&ini_path).unwrap();
        assert!(ini.contains("[reapack]"));
        assert!(ini.contains("version=4"));
        assert!(ini.contains("[remotes]"));
        assert!(ini.contains("size=1"));
        assert!(ini.contains(&format!("remote0={}|{}|1|2", TEST_REPO_NAME, TEST_REPO_URL)));
    }

    #[test]
    fn is_remote_configured_reports_false_when_ini_is_missing() {
        let dir = tempdir().unwrap();
        assert!(!is_remote_configured(dir.path(), TEST_REPO_URL).unwrap());
    }

    #[test]
    fn is_remote_configured_reports_true_when_url_present_in_remotes_section() {
        let dir = tempdir().unwrap();
        let ini_path = dir.path().join(REAPACK_INI_RELATIVE_PATH);
        fs::write(
            &ini_path,
            format!(
                "[remotes]\nsize=1\nremote10={}|{}|1|2\n",
                TEST_REPO_NAME, TEST_REPO_URL
            ),
        )
        .unwrap();
        assert!(is_remote_configured(dir.path(), TEST_REPO_URL).unwrap());
    }

    #[test]
    fn is_remote_configured_reports_false_when_remotes_section_lists_other_urls() {
        let dir = tempdir().unwrap();
        let ini_path = dir.path().join(REAPACK_INI_RELATIVE_PATH);
        fs::write(
            &ini_path,
            "[remotes]\nsize=1\nremote0=ReaTeam Extensions|https://github.com/ReaTeam/Extensions/raw/master/index.xml|1|2\n",
        )
        .unwrap();
        assert!(!is_remote_configured(dir.path(), TEST_REPO_URL).unwrap());
    }

    #[test]
    fn curated_defaults_creates_fresh_file_with_version_and_autoinstall_off() {
        let dir = tempdir().unwrap();
        let outcome = apply_curated_reapack_defaults(dir.path()).unwrap();
        assert_eq!(outcome, CuratedDefaultsOutcome::CreatedFile);

        let ini = fs::read_to_string(dir.path().join(REAPACK_INI_RELATIVE_PATH)).unwrap();
        assert!(ini.contains("[general]"));
        assert!(ini.contains("version=4"));
        assert!(ini.contains("[install]"));
        assert!(ini.contains("autoinstall=0"));
    }

    #[test]
    fn curated_defaults_leaves_a_file_it_already_wrote_untouched() {
        let dir = tempdir().unwrap();
        assert_eq!(
            apply_curated_reapack_defaults(dir.path()).unwrap(),
            CuratedDefaultsOutcome::CreatedFile
        );
        // Second run sees the version marker it wrote and backs off.
        assert_eq!(
            apply_curated_reapack_defaults(dir.path()).unwrap(),
            CuratedDefaultsOutcome::LeftExisting
        );
    }

    #[test]
    fn curated_defaults_never_touches_an_existing_user_config() {
        // The user reinstalls/updates ReaPack via FRABBIT after having
        // deliberately kept ReaPack's default repos and turned autoinstall
        // on. FRABBIT must not remove those repos or flip their settings.
        let dir = tempdir().unwrap();
        let ini_path = dir.path().join(REAPACK_INI_RELATIVE_PATH);
        let original = "[general]\nversion=4\n[install]\nautoinstall=1\n[remotes]\nsize=2\nremote0=ReaPack|https://reapack.com/index.xml|1|1\nremote1=ReaTeam Extensions|https://github.com/ReaTeam/Extensions/raw/master/index.xml|1|1\n";
        fs::write(&ini_path, original).unwrap();

        let outcome = apply_curated_reapack_defaults(dir.path()).unwrap();
        assert_eq!(outcome, CuratedDefaultsOutcome::LeftExisting);

        // Byte-for-byte unchanged: nothing removed, nothing overridden.
        let after = fs::read_to_string(&ini_path).unwrap();
        assert_eq!(after, original);
    }

    #[test]
    fn curated_defaults_leaves_even_a_stale_lower_version_untouched() {
        // An old ReaPack config (version 2) is still the user's own setup,
        // so we do not rewrite it.
        let dir = tempdir().unwrap();
        let ini_path = dir.path().join(REAPACK_INI_RELATIVE_PATH);
        let original = "[general]\nversion=2\n";
        fs::write(&ini_path, original).unwrap();

        assert_eq!(
            apply_curated_reapack_defaults(dir.path()).unwrap(),
            CuratedDefaultsOutcome::LeftExisting
        );
        assert_eq!(fs::read_to_string(&ini_path).unwrap(), original);
    }

    #[test]
    fn curated_defaults_upgrades_a_frabbit_created_remotes_only_file() {
        // Mirrors the real run order: the ReaperAccessible remote step
        // creates reapack.ini (only [remotes], no version), then the
        // curated step runs and adds the safe defaults without disturbing
        // the remote it finds.
        let dir = tempdir().unwrap();
        upsert_remote(dir.path(), TEST_REPO_NAME, TEST_REPO_URL).unwrap();

        let outcome = apply_curated_reapack_defaults(dir.path()).unwrap();
        assert_eq!(outcome, CuratedDefaultsOutcome::Updated);

        let ini = fs::read_to_string(dir.path().join(REAPACK_INI_RELATIVE_PATH)).unwrap();
        assert!(ini.contains("version=4"));
        assert!(ini.contains("autoinstall=0"));
        assert!(ini.contains(&format!("remote0={}|{}|1|2", TEST_REPO_NAME, TEST_REPO_URL)));
        assert!(is_remote_configured(dir.path(), TEST_REPO_URL).unwrap());
    }

    #[test]
    fn extract_scr_lines_returns_only_scr_lines_in_order() {
        let text = "ACT 1 0 \"_RSabc\" \"Custom\" _SWS_ABOUT\r\n\
                    KEY 9 65 _RSabc 0\r\n\
                    SCR 4 0 RSdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef \"Script: foo.lua\" foo.lua\r\n\
                    SCR\t260\t32060\tRScafef00d \"Script: midi.lua\" midi.lua\r\n\
                    # SCR is the keyword we look for\r\n\
                    SCRIPT 1 2 3\r\n";
        let lines = extract_scr_lines(text);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("SCR 4 0 RSdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"));
        assert!(lines[0].contains("foo.lua"));
        assert!(lines[1].starts_with("SCR\t260\t32060"));
    }

    #[test]
    fn extract_scr_lines_handles_empty_and_no_scr_input() {
        assert!(extract_scr_lines("").is_empty());
        assert!(extract_scr_lines("KEY 9 65 40000 0\nACT 1 0 \"_x\" \"y\" 40000\n").is_empty());
    }

    #[test]
    fn reads_package_owner_for_registered_file() {
        let dir = tempdir().unwrap();
        let plugins = dir.path().join("UserPlugins");
        let reapack = dir.path().join("ReaPack");
        fs::create_dir_all(&plugins).unwrap();
        fs::create_dir_all(&reapack).unwrap();
        let sws_file = plugins.join("reaper_sws-x64.dll");
        fs::write(&sws_file, b"").unwrap();

        let connection = Connection::open(reapack.join("registry.db")).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE entries (
                    id INTEGER PRIMARY KEY,
                    remote TEXT NOT NULL,
                    category TEXT NOT NULL,
                    package TEXT NOT NULL,
                    desc TEXT NOT NULL,
                    type INTEGER NOT NULL,
                    version TEXT NOT NULL,
                    author TEXT NOT NULL,
                    flags INTEGER DEFAULT 0,
                    UNIQUE(remote, category, package)
                );
                CREATE TABLE files (
                    id INTEGER PRIMARY KEY,
                    entry INTEGER NOT NULL,
                    path TEXT UNIQUE NOT NULL,
                    main INTEGER NOT NULL,
                    type INTEGER NOT NULL,
                    FOREIGN KEY(entry) REFERENCES entries(id)
                );
                INSERT INTO entries(id, remote, category, package, desc, type, version, author, flags)
                VALUES(1, 'ReaTeam Extensions', 'Extensions', 'SWS/S&M Extension', '', 0, '2.14.0.7', '', 0);
                INSERT INTO files(id, entry, path, main, type)
                VALUES(1, 1, 'UserPlugins/reaper_sws-x64.dll', 1, 0);",
            )
            .unwrap();

        let owner = package_owner_for_file(dir.path(), &sws_file)
            .unwrap()
            .unwrap();
        assert_eq!(owner.package, "SWS/S&M Extension");
        assert_eq!(owner.version.raw(), "2.14.0.7");

        let entries = list_entries(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
    }
}
