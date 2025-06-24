use anyhow::{anyhow, bail, Context, Result};
use dunce::canonicalize;
use rayon::prelude::*;
use std::{
    fs,
    io::{self, BufRead, BufReader},
    path::Path,
    time::SystemTime,
};

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
                return rimraf_impl(&path);
            }
            let rm = if cfg!(windows) && metadata.is_symlink() {
                fs::remove_dir
            } else {
                fs::remove_file
            };
            if let Err(err) = rm(&path) {
                match err.kind() {
                    io::ErrorKind::PermissionDenied => {
                        let mut perm = metadata.permissions();
                        perm.set_readonly(false);
                        fs::set_permissions(&path, perm)?;
                        rm(&path)?;
                    }
                    _ => bail!(err),
                }
            }
            Ok(())
        })
        .collect::<Result<()>>()?;
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

/// Checks if directory exists and is not empty
pub fn is_dir_empty(path: &Path) -> Result<bool> {
    Ok(!path.is_dir() || path.read_dir()?.next().is_none())
}

pub fn set_modified_time(path: impl AsRef<Path>, time: SystemTime) -> Result<()> {
    let inner = || {
        fs::File::options()
            .write(true)
            .open(&path)?
            .set_modified(time)
    };
    inner().context(format!(
        "Failed to set modified time\n\
         <yellow> >></> Path: {}",
        path.as_ref().display(),
    ))
}

#[cfg(unix)]
fn symlink_impl(from: &Path, to: &Path) -> io::Result<()> {
    use std::os::unix;
    unix::fs::symlink(canonicalize(from)?, to)
}

#[cfg(windows)]
fn symlink_impl(from: &Path, to: &Path) -> io::Result<()> {
    use std::os::windows;
    windows::fs::symlink_dir(canonicalize(from)?, to).map_err(|e| match e.raw_os_error() {
        Some(1314) => io::Error::other(
            "A required privilege is not held by the client. (os error 1314)\n\
             <blue>[?]</> Try enabling developer mode in Windows settings or run the terminal as an administrator",
        ),
        _ => e,
    })
}

pub fn symlink(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    symlink_impl(from, to).context(format!(
        "Failed to create symlink\n\
         <yellow> >></> From: {}\n\
         <yellow> >></> To: {}",
        from.display(),
        to.display()
    ))
}

pub fn try_symlink(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    if let Err(e) = symlink(&from, &to) {
        match e.downcast_ref::<io::Error>().map(|e| e.kind()) {
            Some(io::ErrorKind::NotFound) => {
                fs::create_dir_all(&from)?;
                symlink(from, to)?;
            }
            _ => return Err(e),
        }
    }
    Ok(())
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

/// Sync target directory with source directory.
pub fn sync_dir(source: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<()> {
    let source = source.as_ref();
    let target = target.as_ref();
    if target.is_dir() {
        sync_dir_impl(source, target).context(format!(
            "Failed to copy directory\n\
             <yellow> >></> From: {}\n\
             <yellow> >></> To: {}",
            source.display(),
            target.display(),
        ))?;
        cleanup(source, target)
    } else {
        copy_dir(source, target)
    }
}

fn sync_dir_impl(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    fs::read_dir(source)?
        .par_bridge()
        .map(|entry| -> Result<()> {
            let entry = entry?;
            let source = entry.path();
            let target = target.join(entry.file_name());
            if source.is_dir() {
                if target.is_file() {
                    fs::remove_file(&target)?;
                }
                return sync_dir_impl(&source, &target);
            }
            if target.is_dir() {
                rimraf(&target)?;
            }
            if !diff(&source, &target)? {
                fs::copy(source, target)?;
            }
            Ok(())
        })
        .collect()
}

/// Remove files that are not present in the source directory.
fn cleanup(source: &Path, target: &Path) -> Result<()> {
    fs::read_dir(target)?
        .par_bridge()
        .map(|entry| -> Result<()> {
            let entry = entry?;
            let source = source.join(entry.file_name());
            let target = entry.path();
            let is_dir = target.is_dir();
            if !source.exists() {
                if is_dir {
                    rimraf(target)?;
                } else {
                    fs::remove_file(&target).context(format!(
                        "Failed to remove file\n\
                         <yellow> >></> Path: {}",
                        target.display(),
                    ))?;
                }
            } else if is_dir {
                cleanup(&source, &target)?;
            }
            Ok(())
        })
        .collect()
}

/// Compare two file contents. Return true if they are identical.
fn diff(a: &Path, b: &Path) -> Result<bool> {
    let a = fs::File::open(a);
    let b = fs::File::open(b);
    if a.is_err() || b.is_err() {
        return Ok(false);
    }
    let mut a_reader = BufReader::new(a.unwrap());
    let mut b_reader = BufReader::new(b.unwrap());
    if a_reader.capacity() != b_reader.capacity() {
        return Ok(false);
    }
    loop {
        let len = {
            let a_buf = a_reader.fill_buf()?;
            let b_buf = b_reader.fill_buf()?;
            if a_buf.is_empty() && b_buf.is_empty() {
                return Ok(true);
            }
            if a_buf != b_buf {
                return Ok(false);
            }
            a_buf.len()
        };
        a_reader.consume(len);
        b_reader.consume(len);
    }
}
