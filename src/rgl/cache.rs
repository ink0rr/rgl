use crate::fs::{copy_dir, rimraf, try_symlink};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::{
    fs,
    io::{BufRead, BufReader},
    path::Path,
};

pub fn copy_dir_cached(
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
    cache: impl AsRef<Path>,
) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    let cache = cache.as_ref();
    if !to.is_symlink() {
        try_symlink(cache, to)?;
    }
    if cache.is_dir() {
        copy_cached(from, cache).context(format!(
            "Failed to copy directory\n\
             <yellow> >></> From: {}\n\
             <yellow> >></> To: {}",
            from.display(),
            cache.display(),
        ))?;
        cleanup(from, cache)
    } else {
        copy_dir(from, cache)
    }
}

/// Copy files from the source directory to the target directory, but only if they are different.
fn copy_cached(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    fs::read_dir(from)?
        .par_bridge()
        .map(|entry| -> Result<()> {
            let entry = entry?;
            let from = entry.path();
            let to = to.join(entry.file_name());
            if from.is_dir() {
                if to.is_file() {
                    fs::remove_file(&to)?;
                }
                return copy_cached(&from, &to);
            }
            if to.is_dir() {
                rimraf(&to)?;
            }
            if !diff(&from, &to)? {
                fs::copy(from, to)?;
            }
            Ok(())
        })
        .collect()
}

/// Remove files that are not present in the source directory.
fn cleanup(from: &Path, to: &Path) -> Result<()> {
    fs::read_dir(to)?
        .par_bridge()
        .map(|entry| -> Result<()> {
            let entry = entry?;
            let from = from.join(entry.file_name());
            let to = entry.path();
            let is_dir = to.is_dir();
            if !from.exists() {
                return match is_dir {
                    true => rimraf(&to),
                    false => fs::remove_file(&to).context(format!(
                        "Failed to remove file\n\
                         <yellow> >></> Path: {}",
                        to.display(),
                    )),
                };
            }
            if is_dir {
                cleanup(&from, &to)?;
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
