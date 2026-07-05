use super::{
    find_mojang_dir, find_world_dir, get_current_dir, get_dot_regolith_dir, Eval, MinecraftBuild,
};
use anyhow::{anyhow, bail, Result};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Component, Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "target")]
#[enum_dispatch]
pub enum Export {
    Development(DevelopmentExport),
    Local(LocalExport),
    Exact(ExactExport),
    None(NoneExport),
    World(WorldExport),
}

#[enum_dispatch(Export)]
pub trait ExportPaths {
    fn get_paths(&self, project_name: &str, profile_name: &str) -> Result<(PathBuf, PathBuf)>;
}

impl Export {
    /// Whether this target is the `none` export target, which is skipped
    /// during exporting.
    pub fn is_none(&self) -> bool {
        matches!(self, Export::None(_))
    }

    /// Short name used in log messages and error labels.
    pub fn kind(&self) -> &'static str {
        match self {
            Export::Development(_) => "development",
            Export::Local(_) => "local",
            Export::Exact(_) => "exact",
            Export::None(_) => "none",
            Export::World(_) => "world",
        }
    }

    /// Whether the exported files should be made read-only after export.
    /// Mirrors Go's `ExportTarget.ReadOnly`, which is available on every
    /// export target type.
    pub fn read_only(&self) -> bool {
        match self {
            Export::Development(e) => e.read_only,
            Export::Local(e) => e.read_only,
            Export::Exact(e) => e.read_only,
            Export::None(e) => e.read_only,
            Export::World(e) => e.read_only,
        }
    }
}

/// A profile's `export` value. Accepts either a single export target object
/// (the historical, and still dominant, form) or an array of export target
/// objects, in which case the built packs are exported to every target.
///
/// This mirrors Regolith's `ExportTargets`, which also accepts both forms
/// under the same `export` key (see `ExportTargetsFromObject` in
/// `regolith/config.go`), rather than gating the array form behind a
/// separate key or format version.
pub struct ExportTargets(pub Vec<Export>);

impl ExportTargets {
    pub fn targets(&self) -> &[Export] {
        &self.0
    }

    /// Export targets that aren't `none`, in declaration order.
    pub fn active(&self) -> impl Iterator<Item = (usize, &Export)> {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, export)| !export.is_none())
    }
}

impl From<Export> for ExportTargets {
    fn from(export: Export) -> Self {
        ExportTargets(vec![export])
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ExportTargetsRepr {
    Single(Export),
    Multiple(Vec<Export>),
}

impl<'de> Deserialize<'de> for ExportTargets {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let targets = match ExportTargetsRepr::deserialize(deserializer)? {
            ExportTargetsRepr::Single(export) => vec![export],
            ExportTargetsRepr::Multiple(exports) => {
                if exports.is_empty() {
                    return Err(serde::de::Error::custom(
                        "The \"export\" array must contain at least one entry",
                    ));
                }
                exports
            }
        };
        Ok(ExportTargets(targets))
    }
}

impl Serialize for ExportTargets {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Keep single-target profiles serialized as a plain object so
        // configs written by rgl stay backward compatible with tools that
        // only understand the historical single `export` object.
        match self.0.as_slice() {
            [export] => export.serialize(serializer),
            exports => exports.serialize(serializer),
        }
    }
}

/// Resolves `path` to an absolute, lexically-cleaned path (canonicalized
/// when possible) so it can be compared for export path collisions the same
/// way regardless of how it was originally written (relative, with `./`,
/// symlinked, etc).
fn normalize_export_path_for_collision(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        get_current_dir()?.join(path)
    };
    let cleaned = clean_path(&absolute);
    Ok(dunce::canonicalize(&cleaned).unwrap_or(cleaned))
}

/// Lexically removes `.` and `..` components without touching the
/// filesystem (the path may not exist yet).
fn clean_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => match result.components().next_back() {
                Some(Component::Normal(_)) => {
                    result.pop();
                }
                _ => result.push(component),
            },
            other => result.push(other),
        }
    }
    result
}

fn path_contains(parent: &Path, child: &Path) -> bool {
    parent != child && child.strip_prefix(parent).is_ok()
}

/// Checks whether `path` (labeled `label`) collides with any previously
/// seen export path, i.e. resolves to the same location or is nested inside
/// (or contains) a previously seen export path. Mirrors Go's
/// `checkExportPathCollision`.
pub fn check_export_path_collision(
    seen: &mut Vec<(PathBuf, String)>,
    path: &Path,
    label: &str,
) -> Result<()> {
    let normalized = normalize_export_path_for_collision(path)?;
    for (seen_path, seen_label) in seen.iter() {
        if normalized == *seen_path
            || path_contains(&normalized, seen_path)
            || path_contains(seen_path, &normalized)
        {
            bail!(
                "Export path collision detected\n\
                 <yellow> >></> First path: {seen_label}\n\
                 <yellow> >></> Second path: {label}\n\
                 <yellow> >></> Overlapping path: {}",
                normalized.display()
            );
        }
    }
    seen.push((normalized, label.to_owned()));
    Ok(())
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevelopmentExport {
    #[serde(skip_serializing_if = "Option::is_none")]
    build: Option<MinecraftBuild>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bp_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rp_name: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    read_only: bool,
}

impl ExportPaths for DevelopmentExport {
    fn get_paths(&self, project_name: &str, profile_name: &str) -> Result<(PathBuf, PathBuf)> {
        let mojang_dir = find_mojang_dir(self.build.as_ref())?;
        if !mojang_dir.exists() {
            bail!("Failed to find com.mojang directory")
        }
        let eval = Eval::new(profile_name, &get_current_dir()?, None);
        let bp = {
            let dir = mojang_dir.join("development_behavior_packs");
            if let Some(bp_name) = &self.bp_name {
                dir.join(eval.string(bp_name)?)
            } else {
                dir.join(format!("{project_name}_bp"))
            }
        };
        let rp = {
            let dir = mojang_dir.join("development_resource_packs");
            if let Some(rp_name) = &self.rp_name {
                dir.join(eval.string(rp_name)?)
            } else {
                dir.join(format!("{project_name}_rp"))
            }
        };
        Ok((bp, rp))
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalExport {
    #[serde(skip_serializing_if = "Option::is_none")]
    bp_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rp_name: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    read_only: bool,
}

impl ExportPaths for LocalExport {
    fn get_paths(&self, project_name: &str, profile_name: &str) -> Result<(PathBuf, PathBuf)> {
        let build = PathBuf::from("build");
        if !build.exists() {
            fs::create_dir(&build)?;
        }
        let eval = Eval::new(profile_name, &get_current_dir()?, None);
        let bp = if let Some(bp_name) = &self.bp_name {
            build.join(eval.string(bp_name)?)
        } else {
            build.join(format!("{project_name}_bp"))
        };
        let rp = if let Some(rp_name) = &self.rp_name {
            build.join(eval.string(rp_name)?)
        } else {
            build.join(format!("{project_name}_rp"))
        };
        Ok((bp, rp))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExactExport {
    bp_path: String,
    rp_path: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    read_only: bool,
}

impl ExportPaths for ExactExport {
    fn get_paths(&self, _project_name: &str, _profile_name: &str) -> Result<(PathBuf, PathBuf)> {
        let bp = resolve_path(&self.bp_path)?;
        let rp = resolve_path(&self.rp_path)?;
        if bp == rp {
            bail!("Both `bpPath` and `rpPath` resolved to the same path")
        }
        Ok((bp, rp))
    }
}

fn resolve_path(path: &str) -> Result<PathBuf> {
    let mut res = PathBuf::new();
    for component in PathBuf::from(path).components() {
        match component {
            Component::Normal(os_str) => {
                let part = os_str.to_string_lossy();
                if part.starts_with('%') && part.ends_with('%') {
                    let name = &part[1..part.len() - 1];
                    let value = env::var(name)
                        .map_err(|_| anyhow!("Environment variable <b>{name}</> not found"))?;
                    res.push(value);
                } else {
                    res.push(os_str);
                }
            }
            _ => res.push(component),
        }
    }
    fs::create_dir_all(&res).map_err(|_| {
        anyhow!(
            "Cannot create directory when a file with the same name exists\n\
             <yellow> >></> Path: {}",
            res.display()
        )
    })?;
    if res.join("config.json").is_file() {
        bail!(
            "The specified path is a project directory\n\
             <yellow> >></> Path: {}",
            res.display()
        )
    }
    Ok(dunce::canonicalize(res)?)
}

#[derive(Serialize, Deserialize)]
pub struct NoneExport {
    #[serde(
        default,
        rename = "readOnly",
        skip_serializing_if = "std::ops::Not::not"
    )]
    read_only: bool,
}

impl ExportPaths for NoneExport {
    fn get_paths(&self, _project_name: &str, _profile_name: &str) -> Result<(PathBuf, PathBuf)> {
        // Set the export target to temp just to not mess up the log messages
        let temp = get_dot_regolith_dir()?.join("tmp");
        Ok((temp.join("BP"), temp.join("RP")))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldExport {
    #[serde(skip_serializing_if = "Option::is_none")]
    build: Option<MinecraftBuild>,
    #[serde(skip_serializing_if = "Option::is_none")]
    world_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    world_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bp_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rp_name: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    read_only: bool,
}

impl ExportPaths for WorldExport {
    fn get_paths(&self, project_name: &str, profile_name: &str) -> Result<(PathBuf, PathBuf)> {
        let world_dir = match (&self.world_name, &self.world_path) {
            (Some(world_name), None) => find_world_dir(self.build.as_ref(), world_name)?,
            (None, Some(world_path)) => resolve_path(world_path)?,
            (Some(_), Some(_)) => bail!("Using both `worldName` and `worldPath` is not allowed"),
            (None, None) => bail!(
                "The `world` export target requires either a `worldName` or `worldPath property`"
            ),
        };
        let eval = Eval::new(profile_name, &get_current_dir()?, None);
        let bp = {
            let dir = world_dir.join("behavior_packs");
            if let Some(bp_name) = &self.bp_name {
                dir.join(eval.string(bp_name)?)
            } else {
                dir.join(format!("{project_name}_bp"))
            }
        };
        let rp = {
            let dir = world_dir.join("resource_packs");
            if let Some(rp_name) = &self.rp_name {
                dir.join(eval.string(rp_name)?)
            } else {
                dir.join(format!("{project_name}_rp"))
            }
        };
        Ok((bp, rp))
    }
}
