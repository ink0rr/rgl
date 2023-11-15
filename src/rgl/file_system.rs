use anyhow::{anyhow, Context, Result};
use dunce::canonicalize;
use rayon::prelude::*;
use std::{fs, io, path::Path};

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

fn empty_dir_impl(path: &Path) -> Result<()> {
    rimraf(path).map_err(|e| anyhow!("{}", e.root_cause()))?;
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn empty_dir(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    empty_dir_impl(path).context(format!(
        "Failed to empty directory\n\
         <yellow> >></> Path: {}",
        path.display(),
    ))
}

pub fn move_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    fs::rename(from, to).context(format!(
        "Failed to move directory\n\
         <yellow> >></> From: {}\n\
         <yellow> >></> To: {}",
        from.display(),
        to.display(),
    ))
}

pub fn read_json<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let inner = || -> Result<T> {
        let data = fs::read_to_string(&path)?;
        let json = serde_json::from_str(&data)?;
        Ok(json)
    };
    inner().context(format!(
        "Failed to read JSON file {}",
        path.as_ref().display()
    ))
}

fn rimraf_impl(path: &Path) -> Result<()> {
    fs::read_dir(path)?
        .par_bridge()
        .map(|entry| -> Result<()> {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                rimraf_impl(&path)?;
            } else if cfg!(windows) && metadata.is_symlink() {
                fs::remove_dir(path)?;
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

#[cfg(unix)]
fn symlink_impl(from: &Path, to: &Path) -> io::Result<()> {
    use std::os::unix;
    unix::fs::symlink(from, to)
}

#[cfg(windows)]
fn symlink_impl(from: &Path, to: &Path) -> io::Result<()> {
    use std::os::windows;
    windows::fs::symlink_dir(from, to)
}

pub fn symlink(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    let from = canonicalize(from)?;
    let to = to.as_ref();
    symlink_impl(&from, to).context(format!(
        "Failed to create symlink\n\
         <yellow> >></> From: {}\n\
         <yellow> >></> To: {}",
        from.display(),
        to.display()
    ))
}

pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    let path = path.as_ref();
    fs::write(path, contents).context(format!(
        "Failed to write file\n\
         <yellow> >></> Path: {}",
        path.display()
    ))
}

pub fn write_json<T>(path: impl AsRef<Path>, data: &T) -> Result<()>
where
    T: serde::Serialize,
{
    let path = path.as_ref();
    let inner = || -> Result<()> {
        let data = serde_json::to_string_pretty(data)?;
        write_file(path, data + "\n")?;
        Ok(())
    };
    inner().context(format!(
        "Failed to write JSON file\n\
         <yellow> >></> Path: {}",
        path.display()
    ))
}
