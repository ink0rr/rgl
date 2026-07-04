use super::{get_current_dir, get_dot_regolith_dir, UserConfig};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct Temp {
    pub bp: PathBuf,
    pub rp: PathBuf,
    pub data: PathBuf,
    pub root: PathBuf,
}

impl Temp {
    pub fn from_dot_regolith() -> Result<Self> {
        let temp = get_working_dir()?;
        Ok(Self {
            bp: temp.join("BP"),
            rp: temp.join("RP"),
            data: temp.join("data"),
            root: temp,
        })
    }
}

/// Returns the absolute path to the directory where rgl builds filters,
/// mirroring Regolith's `GetAbsoluteWorkingDirectory`.
///
/// By default this is `<dot_regolith_dir>/tmp`. When the user config option
/// `tmp_dir` is set, it's used instead: if it's a relative path it's resolved
/// relative to the dot-regolith directory (like the default `tmp` folder
/// name), and if it's an absolute path, a subdirectory keyed by a hash of the
/// current project's path is used inside of it, to avoid collisions between
/// multiple projects using the same custom tmp_dir.
fn get_working_dir() -> Result<PathBuf> {
    match UserConfig::tmp_dir() {
        None => Ok(get_dot_regolith_dir()?.join("tmp")),
        Some(tmp_dir) => {
            let tmp_dir = Path::new(&tmp_dir);
            if tmp_dir.is_absolute() {
                let project_dir = get_current_dir()?;
                let hash = format!(
                    "{:x}",
                    md5::compute(project_dir.to_string_lossy().as_bytes())
                );
                Ok(tmp_dir.join(hash))
            } else {
                Ok(get_dot_regolith_dir()?.join(tmp_dir).join("tmp"))
            }
        }
    }
}
