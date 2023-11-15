use anyhow::{anyhow, Context, Result};
use dunce::canonicalize;
use rayon::prelude::*;
use std::{fs, path::Path};

fn copy_dir_impl(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    fs::read_dir(from)?
        .par_bridge()
        .map(|entry| -> Result<()> {
            let entry = entry?;
            let path = entry.path();
            let to = to.join(entry.file_name());
            if path.is_dir() {
                copy_dir_impl(&path, &to)?;
            } else {
                fs::copy(path, to)?;
            }
            Ok(())
        })
        .collect()
}

pub fn copy_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    copy_dir_impl(from, to).context(format!(
        "Failed to copy directory\n\
         <yellow> >></> From: {}\n\
         <yellow> >></> To: {}",
        from.display(),
        to.display(),
    ))
}

fn empty_dir_impl(path: impl AsRef<Path>) -> Result<()> {
    rimraf(&path).map_err(|e| anyhow!("{}", e.root_cause()))?;
    fs::create_dir_all(&path)?;
    Ok(())
}

pub fn empty_dir(path: impl AsRef<Path>) -> Result<()> {
    empty_dir_impl(&path).context(format!(
        "Failed to empty directory\n\
         <yellow> >></> Path: {}",
        path.as_ref().display(),
    ))
}

pub fn move_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    fs::rename(&from, &to).context(format!(
        "Failed to move directory\n\
         <yellow> >></> From: {}\n\
         <yellow> >></> To: {}",
        from.as_ref().display(),
        to.as_ref().display(),
    ))
}

fn read_json_impl<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let data = fs::read_to_string(&path)?;
    let json = serde_json::from_str(&data)?;
    Ok(json)
}

pub fn read_json<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    read_json_impl(&path).context(format!(
        "Failed to read JSON file {}",
        path.as_ref().display()
    ))
}

fn rimraf_impl(path: &Path) -> Result<()> {
    let _ = fs::read_dir(path)?
        .par_bridge()
        .map(|entry| -> Result<()> {
            let entry = entry?;
            let path = entry.path();
            if entry.metadata()?.is_dir() {
                rimraf_impl(&path)?;
            } else {
                fs::remove_file(path)?;
            }
            Ok(())
        })
        .collect::<Result<_>>()?;
    fs::remove_dir(path)?;
    Ok(())
}

pub fn rimraf(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        rimraf_impl(path).context(format!(
            "Failed to remove directory\n\
             <yellow> >></> Path: {}",
            path.display()
        ))?;
    }
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn symlink(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    use std::os::unix;

    let from = canonicalize(&from)?;
    unix::fs::symlink(&from, &to).context(format!(
        "Failed to create symlink\n\
         <yellow> >></> From: {}\n\
         <yellow> >></> To: {}",
        from.display(),
        to.as_ref().display()
    ))
}

#[cfg(target_os = "windows")]
pub fn symlink(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    use std::os::windows;

    let from = canonicalize(&from)?;
    windows::fs::symlink_dir(&from, &to).context(format!(
        "Failed to create symlink\n\
         <yellow> >></> From: {}\n\
         <yellow> >></> To: {}",
        from.display(),
        to.as_ref().display()
    ))
}

pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    fs::write(&path, contents).context(format!(
        "Failed to write file\n\
         <yellow> >></> Path: {}",
        path.as_ref().display()
    ))
}

fn write_json_impl<T>(path: impl AsRef<Path>, data: &T) -> Result<()>
where
    T: serde::Serialize,
{
    let data = serde_json::to_string_pretty(data)?;
    write_file(&path, data + "\n")?;
    Ok(())
}

pub fn write_json<T>(path: impl AsRef<Path>, data: &T) -> Result<()>
where
    T: serde::Serialize,
{
    write_json_impl(&path, data).context(format!(
        "Failed to write JSON file\n\
         <yellow> >></> Path: {}",
        path.as_ref().display()
    ))
}
