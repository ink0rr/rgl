use super::{RglError, RglResult};
use dunce::canonicalize;
use serde::de;
use serde_json;
use std::{fs, io, path::Path};

pub fn copy_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> RglResult<()> {
    _copy_dir(&from, &to).map_err(|e| RglError::CopyDir {
        from: from.as_ref().display().to_string(),
        to: to.as_ref().display().to_string(),
        cause: RglError::Wrap(e.into()).into(),
    })
}

fn _copy_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            _copy_dir(entry.path(), to.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), to.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn empty_dir(path: impl AsRef<Path>) -> RglResult<()> {
    rimraf(&path)?;
    fs::create_dir_all(&path).map_err(|e| RglError::EmptyDir {
        path: path.as_ref().display().to_string(),
        cause: RglError::Wrap(e.into()).into(),
    })
}

pub fn move_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> RglResult<()> {
    rimraf(&to)?;
    fs::rename(&from, &to).map_err(|e| RglError::MoveDir {
        from: from.as_ref().display().to_string(),
        to: to.as_ref().display().to_string(),
        cause: RglError::Wrap(e.into()).into(),
    })
}

pub fn read_json<T>(path: impl AsRef<Path>) -> RglResult<T>
where
    T: de::DeserializeOwned,
{
    let data = fs::read_to_string(&path).map_err(|e| RglError::ReadFile {
        path: path.as_ref().display().to_string(),
        cause: RglError::Wrap(e.into()).into(),
    })?;
    serde_json::from_str(&data).map_err(|e| RglError::ReadJson {
        path: path.as_ref().display().to_string(),
        cause: RglError::SerdeJson(e).into(),
    })
}

pub fn rimraf(path: impl AsRef<Path>) -> RglResult<()> {
    if let Err(e) = fs::remove_dir_all(&path) {
        if e.kind() != io::ErrorKind::NotFound {
            return Err(RglError::Rimraf {
                path: path.as_ref().display().to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn symlink(from: impl AsRef<Path>, to: impl AsRef<Path>) -> RglResult<()> {
    use std::os::unix;

    let from = canonicalize(&from).map_err(|e| RglError::Wrap(e.into()))?;
    unix::fs::symlink(&from, &to).map_err(|e| RglError::Symlink {
        from: from.display().to_string(),
        to: to.as_ref().display().to_string(),
        cause: RglError::Wrap(e.into()).into(),
    })
}

#[cfg(target_os = "windows")]
pub fn symlink(from: impl AsRef<Path>, to: impl AsRef<Path>) -> RglResult<()> {
    use std::os::windows;

    let from = canonicalize(&from).map_err(|e| RglError::Wrap(e.into()))?;
    windows::fs::symlink_dir(&from, &to).map_err(|e| RglError::Symlink {
        from: from.display().to_string(),
        to: to.as_ref().display().to_string(),
        cause: RglError::Wrap(e.into()).into(),
    })
}

pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> RglResult<()> {
    fs::write(&path, contents).map_err(|e| RglError::WriteFile {
        path: path.as_ref().display().to_string(),
        cause: RglError::Wrap(e.into()).into(),
    })
}

pub fn write_json<T>(path: impl AsRef<Path>, data: &T) -> RglResult<()>
where
    T: serde::Serialize,
{
    let data = serde_json::to_string_pretty(data).map_err(|e| RglError::WriteJson {
        path: path.as_ref().display().to_string(),
        cause: RglError::SerdeJson(e).into(),
    })?;
    write_file(&path, data + "\n")
}
