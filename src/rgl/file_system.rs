use super::{RglError, RglResult};
use serde::de;
use serde_json;
use std::{fs, io, path::Path};

pub fn copy_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> RglResult<()> {
    if let Err(e) = _copy_dir(&from, &to) {
        return Err(RglError::CopyDir {
            from: from.as_ref().display().to_string(),
            to: to.as_ref().display().to_string(),
            cause: RglError::Wrap(e.into()).into(),
        });
    }
    Ok(())
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
    if let Err(e) = fs::create_dir_all(&path) {
        return Err(RglError::EmptyDir {
            path: path.as_ref().display().to_string(),
            cause: RglError::Wrap(e.into()).into(),
        });
    }
    Ok(())
}

pub fn move_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> RglResult<()> {
    rimraf(&to)?;
    if let Err(e) = fs::rename(&from, &to) {
        return Err(RglError::MoveDir {
            from: from.as_ref().display().to_string(),
            to: to.as_ref().display().to_string(),
            cause: RglError::Wrap(e.into()).into(),
        });
    }
    Ok(())
}

pub fn read_json<T>(path: impl AsRef<Path>) -> RglResult<T>
where
    T: de::DeserializeOwned,
{
    match fs::read_to_string(&path) {
        Err(e) => Err(RglError::ReadFile {
            path: path.as_ref().display().to_string(),
            cause: RglError::Wrap(e.into()).into(),
        }),
        Ok(data) => match serde_json::from_str(&data) {
            Err(error) => Err(RglError::ReadJson {
                path: path.as_ref().display().to_string(),
                cause: RglError::SerdeJson(error).into(),
            }),
            Ok(config) => Ok(config),
        },
    }
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

    let from = match from.as_ref().canonicalize() {
        Ok(path) => path,
        Err(_) => {
            return Err(RglError::PathNotExists {
                path: from.as_ref().display().to_string(),
            })
        }
    };

    if let Err(e) = unix::fs::symlink(&from, &to) {
        return Err(RglError::Symlink {
            from: from.display().to_string(),
            to: to.as_ref().display().to_string(),
            cause: RglError::Wrap(e.into()).into(),
        });
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn symlink(from: impl AsRef<Path>, to: impl AsRef<Path>) -> RglResult<()> {
    use std::os::windows;

    let from = match from.as_ref().canonicalize() {
        Ok(path) => path,
        Err(_) => {
            return Err(RglError::PathNotExists {
                path: from.as_ref().display().to_string(),
            })
        }
    };

    if let Err(e) = windows::fs::symlink_dir(&from, &to) {
        return Err(RglError::Symlink {
            from: from.display().to_string(),
            to: to.as_ref().display().to_string(),
            cause: RglError::Wrap(e.into()).into(),
        });
    }
    Ok(())
}
