use super::get_dot_regolith_dir;
use crate::fs::{read_json, write_json};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use walkdir::WalkDir;

/// Path (relative to the dot-regolith dir) of the cache file that records
/// which files rgl exported to each target on the last successful run.
/// Mirrors Go's `EditedFilesPath`.
const EDITED_FILES_PATH: &str = "cache/edited_files.json";

/// Tracks, per export target path, the list of files rgl created there on
/// the last successful export, so the next export can tell its own files
/// apart from ones a user (or another tool) put there by hand. Mirrors Go's
/// `EditedFiles` (and the on-disk `edited_files.json` it reads/writes).
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EditedFiles {
    #[serde(default)]
    rp: HashMap<String, Vec<String>>,
    #[serde(default)]
    bp: HashMap<String, Vec<String>>,
}

impl EditedFiles {
    /// Loads `edited_files.json` from the dot-regolith cache dir, or falls
    /// back to an empty object if it's missing or fails to parse. Mirrors
    /// Go's `LoadEditedFiles`.
    pub fn load() -> Self {
        let Ok(dot_regolith) = get_dot_regolith_dir() else {
            return Self::default();
        };
        read_json(dot_regolith.join(EDITED_FILES_PATH)).unwrap_or_default()
    }

    /// Writes the current state to `edited_files.json`, creating the cache
    /// directory if needed. Mirrors Go's `EditedFiles.Dump`.
    pub fn dump(&self) -> Result<()> {
        let path = get_dot_regolith_dir()?.join(EDITED_FILES_PATH);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_json(path, self)
    }

    /// Checks whether it's safe to overwrite/delete the current contents of
    /// `rp_path` and `bp_path`, i.e. every file rgl finds there right now is
    /// one it previously exported. Errors (naming the offending file) if it
    /// finds one it doesn't recognize. Mirrors Go's
    /// `EditedFiles.CheckDeletionSafety`.
    pub fn check_deletion_safety(&self, rp_path: &Path, bp_path: &Path) -> Result<()> {
        let empty = Vec::new();
        check_deletion_safety(rp_path, self.rp.get(&path_key(rp_path)).unwrap_or(&empty))
            .context("Deletion safety check for resource pack failed")?;
        check_deletion_safety(bp_path, self.bp.get(&path_key(bp_path)).unwrap_or(&empty))
            .context("Deletion safety check for behavior pack failed")?;
        Ok(())
    }

    /// Records every file currently found under `rp_path`/`bp_path` as
    /// "created by rgl", so they're recognized as safe to delete on the next
    /// export. Mirrors Go's `EditedFiles.UpdateFromPaths`.
    pub fn update_from_paths(&mut self, rp_path: &Path, bp_path: &Path) -> Result<()> {
        let rp_files = list_files(rp_path).with_context(|| {
            format!(
                "Failed to list resource pack files\n\
             <yellow> >></> Path: {}",
                rp_path.display()
            )
        })?;
        let bp_files = list_files(bp_path).with_context(|| {
            format!(
                "Failed to list behavior pack files\n\
             <yellow> >></> Path: {}",
                bp_path.display()
            )
        })?;
        self.rp.insert(path_key(rp_path), rp_files);
        self.bp.insert(path_key(bp_path), bp_files);
        Ok(())
    }
}

/// The key used to look up a path's entry in the `rp`/`bp` maps. Mirrors
/// Go's behavior of using the export path string as-is (no canonicalization)
/// as the map key.
fn path_key(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// `/`-separated form of a path relative to some root, for storage in
/// `edited_files.json` independent of the host OS's path separator. Mirrors
/// Go's `normalizedRelPath` (`strings.ReplaceAll(relpath, "\\", "/")`).
fn to_slash(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Lists every file (not directory) under `path`, as `/`-separated paths
/// relative to `path`. Returns an empty list if `path` doesn't exist.
/// Mirrors Go's `listFiles`.
fn list_files(path: &Path) -> Result<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }
        let relative = entry.path().strip_prefix(path)?;
        result.push(to_slash(relative));
    }
    Ok(result)
}

/// Checks whether it's safe to delete files under `path`, given the list of
/// files known (previously recorded) as safe to delete there. Mirrors Go's
/// `checkDeletionSafety`.
fn check_deletion_safety(path: &Path, removable_files: &[String]) -> Result<()> {
    if !path.exists() {
        return Ok(()); // Directory doesn't exist, nothing to check.
    }
    if !path.is_dir() {
        anyhow::bail!(
            "Path is not a directory\n<yellow> >></> Path: {}",
            path.display()
        );
    }
    let removable: std::collections::HashSet<&str> =
        removable_files.iter().map(String::as_str).collect();
    for entry in WalkDir::new(path).sort_by_file_name() {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }
        let relative = entry.path().strip_prefix(path)?;
        let normalized = to_slash(relative);
        if !removable.contains(normalized.as_str()) {
            anyhow::bail!(
                "File is not on the list of files created by rgl\n\
                 <yellow> >></> Path: {}",
                normalized
            );
        }
    }
    Ok(())
}
