use anyhow::{Context, Error, Result};
use dunce::canonicalize;
use std::{fs, io, path::Path};

fn copy_dir_impl(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            copy_dir_impl(entry.path(), to.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), to.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn copy_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    copy_dir_impl(&from, &to).context(format!(
        "Failed to copy directory\n\
         <yellow> >></> From: {}\n\
         <yellow> >></> To: {}",
        from.as_ref().display(),
        to.as_ref().display(),
    ))
}

fn empty_dir_impl(path: impl AsRef<Path>) -> Result<()> {
    rimraf(&path)?;
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

pub fn rimraf(path: impl AsRef<Path>) -> Result<()> {
    if let Err(e) = fs::remove_dir_all(&path) {
        if e.kind() != io::ErrorKind::NotFound {
            let e = Error::new(e);
            return Err(e.context(format!(
                "Failed to remove directory {}",
                path.as_ref().display()
            )));
        }
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
