use super::{Result, RglError};
use serde::de;
use serde_json;
use std::{fs, io, path::Path};

pub fn copy_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    if let Err(e) = _copy_dir(&from, &to) {
        return Err(RglError::CopyError(
            from.as_ref().display().to_string(),
            to.as_ref().display().to_string(),
            e.to_string(),
        ));
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

pub fn empty_dir(path: impl AsRef<Path>) -> Result<()> {
    rimraf(&path)?;
    if let Err(e) = fs::create_dir_all(&path) {
        return Err(RglError::EmptyDirError(
            path.as_ref().display().to_string(),
            e.to_string(),
        ));
    }
    Ok(())
}

pub fn move_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    rimraf(&to)?;
    if let Err(e) = fs::rename(&from, &to) {
        return Err(RglError::MoveError(
            from.as_ref().display().to_string(),
            to.as_ref().display().to_string(),
            e.to_string(),
        ));
    }
    Ok(())
}

pub fn read_json<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: de::DeserializeOwned,
{
    match fs::read_to_string(&path) {
        Err(e) => Err(RglError::ReadFileError(e.to_string())),
        Ok(data) => match serde_json::from_str(&data) {
            Err(e) => Err(RglError::ReadJsonError(
                path.as_ref().display().to_string(),
                e,
            )),
            Ok(config) => Ok(config),
        },
    }
}

pub fn rimraf(path: impl AsRef<Path>) -> Result<()> {
    if let Err(e) = fs::remove_dir_all(&path) {
        if e.kind() != io::ErrorKind::NotFound {
            return Err(RglError::RimrafError(path.as_ref().display().to_string()));
        }
    }
    Ok(())
}

#[cfg(any(target_os = "windows"))]
pub fn symlink(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    use std::os::windows;

    let from = match from.as_ref().canonicalize() {
        Ok(path) => path,
        Err(_) => {
            return Err(RglError::PathNotExistsError(
                from.as_ref().display().to_string(),
            ))
        }
    };

    if let Err(e) = windows::fs::symlink_dir(&from, &to) {
        return Err(RglError::SymlinkError(
            from.display().to_string(),
            to.as_ref().display().to_string(),
            e.to_string(),
        ));
    }
    Ok(())
}

pub fn write_json<T>(path: impl AsRef<Path>, data: &T) -> Result<()>
where
    T: serde::Serialize,
{
    match fs::write(&path, serde_json::to_string_pretty(data).unwrap()) {
        Err(e) => Err(RglError::WriteFileError(e.to_string())),
        Ok(_) => Ok(()),
    }
}
